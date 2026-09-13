#!/usr/bin/env python3
"""Generate the .rpa test fixtures and the expectations that go with them.

Written against the documented format rather than against renzip's own
writer, on purpose: an archive produced by the code under test only proves
that the code agrees with itself. The layout here matches what
renpy/loader.py reads, so a test that passes on these files is a test
against the engine's format.

Run from anywhere:

    python scripts/make_fixtures.py

It rewrites tests/fixtures/ in place, expected.json included.
"""

import json
import pickle
import zlib
from pathlib import Path

from make_python2_fixture import MEMBERS as PYTHON2_MEMBERS

FIXTURES = Path(__file__).resolve().parent.parent / "tests" / "fixtures"

DEFAULT_KEY = 0xDEADBEEF

# One member with every byte value, so a test that gets encodings wrong on
# the way in or out has somewhere to fail.
BINARY_BLOB = bytes(range(256))


def header_line(version: str, offset: int, key: int) -> bytes:
    """Render the header line for a version, at a fixed width.

    Args:
        version: One of RPA-2.0, RPA-3.0, RPA-3.2, RPA-4.0.
        offset: Absolute offset of the index.
        key: XOR key, ignored by RPA-2.0.

    Returns:
        The header line, newline included.
    """
    if version == "RPA-2.0":
        return b"RPA-2.0 %016x\n" % offset
    if version == "RPA-3.0":
        return b"RPA-3.0 %016x %08x\n" % (offset, key)
    if version == "RPA-3.2":
        # "F" is the literal middle field seen in an actual RPA-3.2 archive
        # (Zeliria Sanctuary, Shizmob/rpatool#9); it is not a placeholder for
        # something else, the format just carries an unused field there.
        return b"RPA-3.2 %016x F %08x\n" % (offset, key)
    if version == "RPA-4.0":
        # Same layout as RPA-3.0 (Lattyware/unrpa#22).
        return b"RPA-4.0 %016x %08x\n" % (offset, key)
    raise ValueError(f"unknown version {version}")


def write_archive(path, version, members, key=DEFAULT_KEY, index_offset=None):
    """Write one .rpa archive.

    Args:
        path: Destination file.
        version: Header version to write.
        members: Mapping of member name to (content, prefix_length).
        key: XOR key applied to offsets and lengths for RPA-3.x.
        index_offset: Offset written in the header, overriding the real one.
            Used to forge a truncated archive.

    Returns:
        Mapping of member name to its content, for the expectations file.
    """
    if version == "RPA-2.0":
        key = 0

    head = header_line(version, 0, key)
    body = bytearray()
    index = {}
    contents = {}

    for name, (content, prefix_length) in members.items():
        prefix = content[:prefix_length]
        stored = content[prefix_length:]
        offset = len(head) + len(body)
        body += stored
        index[name] = [(offset ^ key, len(content) ^ key, prefix)]
        contents[name] = content

    real_offset = len(head) + len(body)
    payload = zlib.compress(pickle.dumps(index, protocol=2))
    path.write_bytes(header_line(version, index_offset if index_offset is not None else real_offset, key) + bytes(body) + payload)
    return contents


OFFICIAL_KEY = 0x42424242
OFFICIAL_PADDING = b"Made with Ren'Py."


def write_official_style_archive(path, version, members):
    """Write an archive the way Ren'Py's own archiver builds one.

    launcher/game/archiver.rpy (unchanged from 6.99 to 8.5.3) uses a fixed
    key, pads every member with the same banner before its bytes, always
    writes a 3-field tuple with an empty byte-string prefix, and pickles the
    index at the interpreter's highest protocol (5, on any Python that runs
    Ren'Py 8). None of that is exercised by the other fixtures, which favor
    protocol 2 and 2-field tuples for readability; this one exists to check
    this crate against the shape an unmodified Ren'Py 8 install actually
    produces, protocol 5 opcodes included.

    Args:
        path: Destination file.
        version: Header version to write.
        members: Mapping of member name to content.

    Returns:
        Mapping of member name to its content, for the expectations file.
    """
    key = OFFICIAL_KEY
    head = header_line(version, 0, key)
    body = bytearray()
    index = {}
    contents = {}

    for name, content in members.items():
        body += OFFICIAL_PADDING
        offset = len(head) + len(body)
        body += content
        index[name] = [(offset ^ key, len(content) ^ key, b"")]
        contents[name] = content

    real_offset = len(head) + len(body)
    payload = zlib.compress(pickle.dumps(index, protocol=pickle.HIGHEST_PROTOCOL))
    path.write_bytes(header_line(version, real_offset, key) + bytes(body) + payload)
    return contents


V1_PADDING = b"\xaa" * 7


def write_v1_archive(path, members):
    """Write an RPA-1.0 archive: raw data, index in a sibling .rpi file.

    Ren'Py 4.8.7's own archiver.py splits every file into several 1-10240
    byte blocks separated by 1-32 bytes of random padding, one (offset,
    length) tuple per block; the padding sizes are fixed here rather than
    randomized so the fixture stays reproducible byte for byte.

    Args:
        path: Destination .rpa file.
        members: Mapping of member name to content: either raw bytes,
            written as a single block, or a list of byte chunks, each
            written as its own block with padding before it (but the first).

    Returns:
        Mapping of member name to its full (concatenated) content.
    """
    body = bytearray()
    index = {}
    contents = {}
    for name, content in members.items():
        blocks = content if isinstance(content, list) else [content]
        tuples = []
        for i, block in enumerate(blocks):
            if i > 0:
                body += V1_PADDING
            tuples.append((len(body), len(block)))
            body += block
        index[name] = tuples
        contents[name] = b"".join(blocks)
    path.write_bytes(bytes(body))
    path.with_suffix(".rpi").write_bytes(zlib.compress(pickle.dumps(index, protocol=2)))
    return contents


def main() -> None:
    """Regenerate every fixture and write expected.json."""
    FIXTURES.mkdir(parents=True, exist_ok=True)
    expected = {}

    plain = {
        "script.rpy": (b'label start:\n    "Hello."\n    return\n', 0),
        "images/bg.png": (BINARY_BLOB, 0),
        "gui/nested/deep/file.txt": (b"nested content\n", 0),
    }
    for version, name in (
        ("RPA-2.0", "basic-2.0.rpa"),
        ("RPA-3.0", "basic-3.0.rpa"),
        ("RPA-3.2", "basic-3.2.rpa"),
    ):
        contents = write_archive(FIXTURES / name, version, plain)
        expected[name] = {"version": version, "entries": encode(contents)}

    # Seen only in games with a modified loader (Lattyware/unrpa#22), never in
    # a stock Ren'Py install; the key there is small (0x42), unlike the other
    # fixtures' DEFAULT_KEY.
    contents = write_archive(FIXTURES / "basic-4.0.rpa", "RPA-4.0", plain, key=0x42)
    expected["basic-4.0.rpa"] = {"version": "RPA-4.0", "entries": encode(contents)}

    official = {
        "script.rpy": b'label start:\n    "Hello."\n    return\n',
        "images/bg.png": BINARY_BLOB,
    }
    contents = write_official_style_archive(FIXTURES / "official-3.0.rpa", "RPA-3.0", official)
    expected["official-3.0.rpa"] = {"version": "RPA-3.0", "entries": encode(contents)}

    # Written by Ren'Py 6.18's own archiver under its Python 2, not here: see
    # make_python2_fixture.py, which needs that SDK. Only the expectation is
    # written by this script; the committed archive is left as it is.
    expected["renpy6-3.0.rpa"] = {
        "version": "RPA-3.0",
        "entries": encode(dict(PYTHON2_MEMBERS)),
    }

    # Prefixed members: the first bytes of the content live in the index and
    # not in the data area, which is where an off-by-one on length shows up.
    prefixed = {
        "prefixed.rpy": (b"# a prefixed script\nlabel a:\n    return\n", 20),
        "whole-prefix.txt": (b"all of me", 9),
        "no-prefix.txt": (b"none of me", 0),
    }
    contents = write_archive(FIXTURES / "prefix-3.0.rpa", "RPA-3.0", prefixed)
    expected["prefix-3.0.rpa"] = {"version": "RPA-3.0", "entries": encode(contents)}

    hostile = {
        "../../escaped.txt": (b"nope", 0),
        "/etc/passwd": (b"nope", 0),
        "C:\\Windows\\System32\\evil.dll": (b"nope", 0),
        "CON": (b"nope", 0),
        "game/nul.txt": (b"nope", 0),
        "bad<char>.txt": (b"nope", 0),
        "harmless.txt": (b"this one is fine\n", 0),
    }
    write_archive(FIXTURES / "malicious-3.0.rpa", "RPA-3.0", hostile)
    expected["malicious-3.0.rpa"] = {
        "version": "RPA-3.0",
        "entries": encode({"harmless.txt": b"this one is fine\n"}),
        "rejected": sorted(name for name in hostile if name != "harmless.txt"),
    }

    write_archive(FIXTURES / "empty-2.0.rpa", "RPA-2.0", {})
    expected["empty-2.0.rpa"] = {"version": "RPA-2.0", "entries": {}}

    # Truncated: the header points at an index that the file is too short to
    # hold. Opening it must fail cleanly rather than read whatever is there.
    write_archive(FIXTURES / "truncated-3.0.rpa", "RPA-3.0", plain, index_offset=0xFFFFFF)

    # Out of bounds: a well-formed index whose entry reaches past the file.
    forged_key = DEFAULT_KEY
    head = header_line("RPA-3.0", 0, forged_key)
    body = b"short"
    index = {
        "fits.txt": [(len(head) ^ forged_key, 5 ^ forged_key, b"")],
        "runs-past-the-end.txt": [(len(head) ^ forged_key, 0xFFFFFF ^ forged_key, b"")],
    }
    payload = zlib.compress(pickle.dumps(index, protocol=2))
    (FIXTURES / "out-of-bounds-3.0.rpa").write_bytes(
        header_line("RPA-3.0", len(head) + len(body), forged_key) + body + payload
    )
    expected["out-of-bounds-3.0.rpa"] = {
        "version": "RPA-3.0",
        "entries": encode({"fits.txt": b"short"}),
        "rejected": ["runs-past-the-end.txt"],
    }

    # Two 2-field tuples for one name: Ren'Py concatenates them, so does renzip.
    head = header_line("RPA-2.0", 0, 0)
    body = b"firstsecond"
    index = {"twice.txt": [(len(head), 5), (len(head) + 5, 6)]}
    payload = zlib.compress(pickle.dumps(index, protocol=2))
    (FIXTURES / "multi-segment-2.0.rpa").write_bytes(
        header_line("RPA-2.0", len(head) + len(body), 0) + body + payload
    )
    expected["multi-segment-2.0.rpa"] = {
        "version": "RPA-2.0",
        "entries": encode({"twice.txt": b"firstsecond"}),
    }

    # Two 3-field tuples for one name: Ren'Py's loader unpacks each tuple of a
    # multi-tuple entry as (offset, length) and raises ValueError on the third
    # field, so this entry cannot be read either way and is reported rejected.
    head = header_line("RPA-2.0", 0, 0)
    body = b"firstsecond"
    index = {
        "unreadable.txt": [(len(head), 5, b""), (len(head) + 5, 6, b"")],
        "fine.txt": [(len(head) + 11, 4)],
    }
    body += b"fine"
    payload = zlib.compress(pickle.dumps(index, protocol=2))
    (FIXTURES / "rejected-segments-2.0.rpa").write_bytes(
        header_line("RPA-2.0", len(head) + len(body), 0) + body + payload
    )
    expected["rejected-segments-2.0.rpa"] = {
        "version": "RPA-2.0",
        "entries": encode({"fine.txt": b"fine"}),
        "rejected": ["unreadable.txt"],
    }

    contents = write_v1_archive(
        FIXTURES / "legacy-1.0.rpa",
        {
            "old.rpy": b"label old:\n    return\n",
            # Split into several blocks with padding between them, the way
            # Ren'Py 4.8.7's own archiver.py splits every file into blocks
            # separated by random padding (fixed-size here to stay
            # reproducible): this is what proves segments are concatenated
            # rather than only the first one read.
            "data.bin": [BINARY_BLOB[:100], BINARY_BLOB[100:200], BINARY_BLOB[200:]],
        },
    )
    expected["legacy-1.0.rpa"] = {"version": "RPA-1.0", "entries": encode(contents)}

    (FIXTURES / "expected.json").write_text(
        json.dumps(expected, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"wrote {len(expected)} fixtures to {FIXTURES}")


def encode(contents):
    """Render member contents as hex, so expectations survive JSON.

    Args:
        contents: Mapping of member name to raw bytes.

    Returns:
        Mapping of member name to the hex form of its content.
    """
    return {name: data.hex() for name, data in contents.items()}


if __name__ == "__main__":
    main()
