# The RPA archive format

This is what renzip knows about Ren'Py's `.rpa` archives: how the engine
itself reads and writes them, which unofficial variants exist, and the rules
renzip follows as a result. Every statement says where it was checked. It was
written from the engine's source and from reports of real archives; no code
from any other archive tool was reused.

## Layout

An archive holds three things, in this order:

1. a header line (every version but RPA-1.0), which gives the offset of the
   index and, for the obfuscated versions, a key;
2. the stored files, back to back and uncompressed, possibly separated by
   padding;
3. the index, which runs from that offset to the end of the file: a Python
   pickle of a dict, compressed with zlib.

The index maps each file name to a list of tuples. A tuple is
`(offset, length)` or `(offset, length, prefix)`, where `prefix` is a byte
string that opens the file content and is kept in the index rather than in
the data area.

Only the index is compressed. Reading one file is a seek and a read.

## Versions

| Version | Header line | Key | Stock Ren'Py | renzip |
|---|---|---|---|---|
| RPA-1.0 | None: the index lives in a sibling `.rpi` file, which starts with the zlib bytes `78 9C`; the data sits in the `.rpa` | None | Reads | Reads |
| RPA-2.0 | `RPA-2.0 <offset, 16 hex digits>` | None | Reads | Reads, writes |
| RPA-3.0 | `RPA-3.0 <offset, 16 hex digits> <key, 8 hex digits>` | `int(line[25:33], 16)` | Reads | Reads, writes |
| RPA-3.2 | `RPA-3.2 <offset> F <key>` | Fourth field | Does not read | Reads |
| RPA-4.0 | `RPA-4.0 <offset> <key>` | Third field | Does not read | Reads |

For RPA-3.x and RPA-4.0, the offset and the length of every tuple are XORed
with the key; the prefix is not.

The engine takes the key from one fixed slice of the header line. renzip
XORs together every word from the key's position to the end of the line,
which gives the same key for every header the engine writes.

Sources: `renpy/loader.py`, read from Ren'Py 6.18 to 8.5.3 (`RPAv3ArchiveHandler.read_index`,
lines 158-182 of the 8.5.3 SDK; the RPA-1.0 handler, lines 216-236). Folding
extra header words into the key comes from rpatool (commit `5a3b12c`), not
from the engine.

## What Ren'Py's own archiver writes

- **From 6.15 to 8.5.3** (`launcher/game/archiver.rpy`, checked on 6.15,
  6.16, 6.17, 6.99.14, 7.3.5, 7.8.7 and 8.5.3): RPA-3.0 with the fixed key
  `0x42424242`, the bytes `Made with Ren'Py.` before every file, one
  three-field tuple per file with an empty prefix, and the index pickled at
  the interpreter's highest protocol: 2 on the Python 2 releases, up to 7.x,
  and 5 on Ren'Py 8.
- **From 6.7 to 6.14** (`renpy/tools/archiver.py`, read on 6.14): RPA-3.0
  with a random key, 1 to 64 random bytes before every file, and one
  three-field tuple per file with an empty prefix. An empty file gets an
  empty list.
- **4.8.7** (`archiver.py`): RPA-1.0. Every file is split into blocks of 1 to
  10,240 bytes, with 1 to 32 random bytes between blocks, and the index holds
  one two-field tuple per block.

## How the engine reads an entry

From `load_from_archive` in `renpy/loader.py` (lines 574-611 of the 8.5.3
SDK), and from archives built for the purpose and read back by that code:

- **One tuple, no prefix**: the file is `length` bytes read at `offset`.
- **One tuple with a prefix**:
  - up to 7.6 and 8.1, the file is the prefix followed by
    `length - len(prefix)` bytes read at `offset`: `length` covers the
    prefix;
  - from 7.7 and 8.2 (read on 7.8.7 and 8.5.3, run on 8.5.3), the code builds
    the stream but never returns it, so the entry cannot be loaded at all.
- **Several tuples**: the engine concatenates the bytes of every
  `(offset, length)` pair, in order. A three-field tuple among them raises
  `ValueError` (read on 6.14, 6.99.14 and 8.1.3, run on 8.5.3).
- **An empty list**: reading the index raises `IndexError` on the first
  tuple, and the whole archive fails to load (read on 6.14 and 8.5.3).

The engine only turns `.rpa` and `.rpi` files that sit on disk in its search
path into archives (`scandirfiles_from_filesystem`, lines 431-454 of the 8.5.3
SDK). An archive stored inside another archive is never opened.

## The rules renzip follows

- A single tuple's length includes its prefix, the only reading under which
  any engine could load a prefixed entry.
- Several two-field tuples are concatenated. An entry with several tuples
  where one has three fields is refused, as is an archive with an empty list.
- Offsets and lengths are checked against the size of the file, overflow
  included, before anything is read.
- An archive renzip writes is RPA-2.0 or RPA-3.0, with one tuple per file, an
  empty prefix, no padding, and the index pickled at protocol 2, the newest
  one both Python 2 and Python 3 releases read. Its default key is
  `0xDEADBEEF`, which only obfuscates.
- `.rpa` and `.rpi` files are left out of an archive being built.
- The index is decoded without running any pickle code, and inflating it
  stops at 256 MiB.

## Unofficial variants

These headers come from games shipping a modified loader. Stock Ren'Py reads
none of them.

- **RPA-3.2**: seen in Zeliria Sanctuary, whose developer confirmed a custom
  format. The header is `RPA-3.2 <offset> F <key>`, for example
  `RPA-3.2 000000000058accd F 42424240`; the key is the fourth field
  (Shizmob/rpatool#9, Lattyware/unrpa#18). renzip reads it.
- **RPA-4.0**: header `RPA-4.0 0000000027d75111 00000042`, the RPA-3.0
  layout under another name; archives open when read as RPA-3.0
  (Lattyware/unrpa#22). renzip reads it.
- **ALT-1.0**: described by unrpa as the key in the second field, XORed with
  `0xDABE8DF0`, and the offset in the third. It appears in no Ren'Py 6.x
  release. renzip does not read it.
- **ZiX-12A and ZiX-12B**: an obfuscation that needs the game's own
  `loader.pyo` to undo. renzip does not read them, by design.
- **Reported without a sample**: `R4598P5757A 000000009ee33010 42424242`
  (Lattyware/unrpa#39) and `RPA-9.1 12acac407b5c514d` (Lattyware/unrpa#46).

## Sources

- The Ren'Py engine, <https://github.com/renpy/renpy>: `renpy/loader.py`,
  `archiver.py`, `renpy/tools/archiver.py` and `launcher/game/archiver.rpy`
  at the tags named above, and the 8.5.3 SDK (8.5.3.26051504) run locally.
- Shizmob/rpatool: pull request #9, commit `5a3b12c`.
- Lattyware/unrpa: issues #18, #22, #39 and #46, pull request #25.
- The test fixtures in `tests/fixtures/`, including `renpy6-3.0.rpa`, written
  by the archiver of the 6.18.3 SDK under its own Python 2.
