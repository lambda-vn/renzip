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

FIXTURES = Path(__file__).resolve().parent.parent / "tests" / "fixtures"

DEFAULT_KEY = 0xDEADBEEF

# One member with every byte value, so a test that gets encodings wrong on
# the way in or out has somewhere to fail.
BINARY_BLOB = bytes(range(256))


def header_line(version: str, offset: int, key: int) -> bytes:
    """Render the header line for a version, at a fixed width.

    Args:
        version: One of RPA-2.0, RPA-3.0, RPA-3.2.
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


def write_v1_archive(path, members):
    """Write an RPA-1.0 archive: raw data, index in a sibling .rpi file.

    Args:
        path: Destination .rpa file.
        members: Mapping of member name to content.

    Returns:
        Mapping of member name to its content.
    """
    body = bytearray()
    index = {}
    for name, content in members.items():
        index[name] = [(len(body), len(content))]
        body += content
    path.write_bytes(bytes(body))
    path.with_suffix(".rpi").write_bytes(zlib.compress(pickle.dumps(index, protocol=2)))
    return dict(members)


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

    # Two tuples for one name: Ren'Py reads the first, and renzip says so.
    head = header_line("RPA-2.0", 0, 0)
    body = b"firstsecond"
    index = {"twice.txt": [(len(head), 5, b""), (len(head) + 5, 6, b"")]}
    payload = zlib.compress(pickle.dumps(index, protocol=2))
    (FIXTURES / "multi-segment-2.0.rpa").write_bytes(
        header_line("RPA-2.0", len(head) + len(body), 0) + body + payload
    )
    expected["multi-segment-2.0.rpa"] = {
        "version": "RPA-2.0",
        "entries": encode({"twice.txt": b"first"}),
        "multi_segment": ["twice.txt"],
    }

    contents = write_v1_archive(
        FIXTURES / "legacy-1.0.rpa", {"old.rpy": b"label old:\n    return\n", "data.bin": BINARY_BLOB}
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
