#!/usr/bin/env python3
"""Read an .rpa archive close to the way Ren'Py does, and print what it holds.

Used to check the other direction of the loop test: that an archive written
by renzip is read back correctly by an implementation that is not renzip.
This is not a full port of renpy/loader.py - it combines a header's subkeys
by XOR and reads only the first tuple of a single-tuple entry, both of which
match the engine, but it does not reproduce every other thing the loader
does. Pickle module included, which is exactly why this script must only
ever be pointed at archives you produced yourself.

    python scripts/read_archive.py path/to/archive.rpa

Prints a JSON object: {"version": "RPA-3.0", "entries": {"name": "<hex>"}}.
"""

import json
import pickle
import sys
import zlib
from pathlib import Path


def read_index(handle, path):
    """Read the header and the index of an open archive.

    Args:
        handle: The archive, opened in binary mode at its start.
        path: Path of the archive, used to find a .rpi sibling.

    Returns:
        A (version, index) pair, index mapping a name to its tuple list.
    """
    line = handle.readline().rstrip(b"\r\n")
    parts = line.split()
    magic = parts[0].decode("ascii", "replace") if parts else ""

    if magic not in ("RPA-2.0", "RPA-3.0", "RPA-3.2"):
        index_path = path.with_suffix(".rpi")
        if not index_path.is_file():
            raise SystemExit(f"unsupported header: {line!r}")
        return "RPA-1.0", pickle.loads(zlib.decompress(index_path.read_bytes()))

    offset = int(parts[1], 16)
    key = 0
    if magic in ("RPA-3.0", "RPA-3.2"):
        for subkey in parts[3 if magic == "RPA-3.2" else 2:]:
            key ^= int(subkey, 16)

    handle.seek(offset)
    index = pickle.loads(zlib.decompress(handle.read()))
    if key:
        index = {
            name: [(entry[0] ^ key, entry[1] ^ key) + tuple(entry[2:]) for entry in entries]
            for name, entries in index.items()
        }
    return magic, index


def main(argv):
    """Print the content of the archive named on the command line.

    Args:
        argv: Command line arguments, without the program name.
    """
    if len(argv) != 1:
        raise SystemExit(__doc__)

    path = Path(argv[0])
    with path.open("rb") as handle:
        version, index = read_index(handle, path)
        entries = {}
        for name, tuples in sorted(index.items()):
            if len(tuples) == 1:
                offset, length = tuples[0][0], tuples[0][1]
                prefix = tuples[0][2] if len(tuples[0]) > 2 else b""
                if isinstance(prefix, str):
                    prefix = prefix.encode("latin1")
                handle.seek(offset)
                entries[name] = (prefix + handle.read(length - len(prefix))).hex()
            else:
                # Ren'Py concatenates every (offset, length) pair of a
                # multi-tuple entry; unpacking one with a third field the
                # same way raises ValueError there, which this does too.
                data = b""
                for offset, length in tuples:
                    handle.seek(offset)
                    data += handle.read(length)
                entries[name] = data.hex()

    json.dump({"version": version, "entries": entries}, sys.stdout, sort_keys=True)


if __name__ == "__main__":
    main(sys.argv[1:])
