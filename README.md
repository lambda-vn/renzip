<div align="center">
  <img src="src/assets/logo-256.png" alt="" width="96">
  <h1>RenZip</h1>

[![Latest release](https://img.shields.io/github/v/release/lambda-vn/renzip?label=release)](https://github.com/lambda-vn/renzip/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/lambda-vn/renzip/ci.yml?branch=main&label=CI)](https://github.com/lambda-vn/renzip/actions/workflows/ci.yml)
[![Build](https://img.shields.io/github/actions/workflow/status/lambda-vn/renzip/build.yml?label=build)](https://github.com/lambda-vn/renzip/actions/workflows/build.yml)
[![License: CeCILL-2.1](https://img.shields.io/badge/license-CeCILL--2.1-blue)](LICENSE.md)
[![Rust + Tauri 2](https://img.shields.io/badge/rust%20%2B%20tauri-2-orange)](https://v2.tauri.app)
[![Platforms: Windows, macOS, Linux](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/lambda-vn/renzip/releases/latest)

</div>

Desktop archive manager for **Ren'Py `.rpa` files**, in the spirit of 7-Zip or
WinRAR but dedicated to the format visual novels ship their assets in. Open an
archive, see what is inside, extract all of it or three files out of it, and
pack a folder back into one. No command line, no Python script, no `.bat`
downloaded from a forum.

Read this in French: [README_FR.md](README_FR.md).

## Features

- Open an archive by picking it, or by dropping it on the window.
- One tree for the whole archive: folders unfold in place, a tick box on any
  row builds the selection, and ticking a folder takes everything below it.
  The list is virtualised, so fifty thousand entries scroll like ten.
- Search that keeps the full path of what it matched, so a hit three folders
  down is still readable as such.
- Preview of text entries and of the usual image formats, straight from the
  archive, without extracting anything first.
- Row menu: extract this, copy the entry name, open a temporary copy with
  whatever the desktop opens that kind of file with.
- Extract a selection or the whole archive, keeping the archive folders or
  flattening everything side by side, with an honest progress bar and a
  cancel button.
- Build a new archive from a folder, optionally reopening it afterwards to
  check it against what was packed.
- French and English interface, chosen in the settings, guessed from the
  desktop on first launch. Light and dark themes.
- Keyboard-operable throughout, following the ARIA tree pattern: arrows walk
  and unfold, space ticks, Enter opens.

## Format support

An `.rpa` is a container, not a compressor: only its index is compressed, the
file data is stored as is. Selective extraction is therefore a `seek` and a
`read`, which is what makes a preview of one entry inside a four gigabyte
archive instant.

| Version | Read | Write | Note |
|---|---|---|---|
| RPA-1.0 | yes | no | Index sits in a sibling `.rpi` file |
| RPA-2.0 | yes | yes | Plain offsets and lengths |
| RPA-3.0 | yes | yes | Offsets and lengths obfuscated by XOR with a key |
| RPA-3.2 | yes | no | One extra header field before the key |
| RPA-4.0 | yes | no | Same structure as 3.0, only the marker differs |

Neither 3.2 nor 4.0 is offered as a write target. Their header shape is no
longer a mystery - 3.2's is offset, an extra unused field, then the key
(confirmed by the developer of Zeliria Sanctuary, the one known game shipping
it, and by Shizmob/rpatool#9); 4.0's is the same as 3.0's (Lattyware/unrpa#22)
- so this crate could write either. But no standard Ren'Py build reads them at
all: a game that opens one embeds a modified loader, so writing an archive the
engine itself cannot read would be a worse outcome than refusing, whatever the
header says. The settings screen says so rather than hiding the option.

The format itself, how each Ren'Py release reads and writes it, and where each
of those facts was checked are written down in
[docs/rpa-format.md](docs/rpa-format.md).

Two things the format simply does not hold, and that the window therefore does
not show: a checksum and a modification date. Nor a compressed size, since
nothing is compressed.

## Stack

Rust / Tauri 2 / Svelte 5 / TypeScript / Tailwind 4.

The reader and writer live in `renzip-core`, a plain Rust library with no
Tauri dependency at all. That is a hard rule: it keeps the format work
testable without opening a window, and leaves the door open for a command line
binary later.

## Requirements

- Rust stable, 1.88 or newer, the oldest release every dependency accepts.
- Node 20 or newer.
- The Tauri system dependencies for your platform, listed in the
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). On Debian
  and Ubuntu that is WebKitGTK and a handful of development packages; on
  Windows, WebView2, which Windows 11 already has.
- Python 3, only to regenerate the test fixtures and the icon. Nothing in
  the build or in the application needs it.

## Installation

```bash
git clone https://github.com/lambda-vn/renzip.git
cd renzip
npm install
```

## Running

```bash
npm run tauri dev
```

Vite serves the frontend and Tauri opens the window on it. Drop an `.rpa` on
that window, or use the button.

## Development

After every change, all four checks must pass, in the order the CI plays them:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
npm run build
```

`npm run build` is not only a build: it runs `svelte-check` first, which is
the type checker and the accessibility linter of the frontend.

`pre-commit install` wires the same checks except the tests, plus the commit
message hook.

### Tests

Unit tests beside the code, integration tests in
`crates/renzip-core/tests/`. Two of them matter more than the rest:

- `read_fixtures.rs` opens every archive in `tests/fixtures/` and compares
  what comes out against `expected.json`, which was produced by a Python
  script reading the same archives the way Ren'Py does, not by this code.
- `roundtrip.rs` packs a directory, opens the result, extracts it, and checks
  the files are identical byte for byte. That is the test that catches a
  mistake in the `prefix` handling or in a bound.

The fixtures cover an RPA-2.0, an RPA-3.0, an RPA-3.2, an RPA-4.0, a legacy 1.0
archive with files split into several blocks, an archive written the way
Ren'Py's own archiver does, one written by that archiver itself under the
Python 2 of a Ren'Py 6.18 SDK, one whose entries carry a prefix, an empty one, a
truncated one, one declaring bytes past the end of the file, one whose entries
concatenate several segments, one with a segment no reader can make sense of
either, and one full of hostile paths.

```bash
cargo test --workspace
python scripts/make_fixtures.py           # rewrite tests/fixtures/
python scripts/read_archive.py x.rpa      # read an archive with pickle, as Ren'Py does
```

### Building a desktop binary

```bash
npm run tauri build
```

Builds for the system it runs on: Tauri links against the native webview, so
nothing cross-compiles between the three platforms. All three are built by the
Build workflow, one runner each, and macOS as a universal bundle. Binaries are
unsigned, so SmartScreen and Gatekeeper will warn.

## Project layout

```
crates/renzip-core/    The format, with no Tauri dependency
  src/header.rs        Version detection and header parsing
  src/index.rs         zlib and pickle, both ways
  src/archive.rs       Opening, listing, reading, extracting
  src/builder.rs       Scanning a directory and writing an archive
  src/path.rs          Entry path sanitizing and containment
  src/error.rs         Errors that stop an operation
  src/reason.rs        Why an entry is skipped or refused
  tests/               Fixture reading and the roundtrip test
src-tauri/             Commands, window, progress events
  src/error.rs         What a failed command returns to the window
  src/desktop.rs       Handing a file to the desktop, per platform
  src/preview.rs       Deciding what to show for an entry
  src/state.rs         Open archives and their scratch directories
src/                   Svelte 5 frontend
  lib/EntryTree.svelte The one list of the main window
  lib/VirtualList.svelte  Hand-written virtualisation, no dependency
  lib/i18n.svelte.ts   French and English
docs/                  The format, and where each fact was checked
scripts/               Fixtures, icon, commit message hook
tests/fixtures/        Archives to read, and what they must produce
```

## Security

A `.rpa` is a file of unknown origin, and this application treats it as
hostile. The short version:

- The index is a Python pickle. Parsed by `serde-pickle`, which reads values
  and never constructs an object the stream asks for, and accepted only in
  the one shape the format defines.
- Entry paths are sanitised, then the destination is checked on the canonical
  path just before writing. A refused entry is reported and skipped, never
  fatal to the rest of the extraction.
- Offsets and lengths are validated against the real file size before
  anything is allocated. Nothing loads an archive whole.
- The window has no command that takes a path and opens it.

The full policy, and what is deliberately outside it, is in
[SECURITY.md](SECURITY.md).

## Commits

Conventional Commits, validated by pre-commit. See
[COMMIT_CONVENTION.md](COMMIT_CONVENTION.md) for types, scopes, and examples.

## License

Licensed under the CeCILL Free Software License Agreement v2.1, a French
open source license compatible with the GNU GPL. See [LICENSE.md](LICENSE.md).

## Disclaimer

RenZip is not affiliated with, endorsed by, or sponsored by Ren'Py. The name
is used only to designate the engine whose archive format this tool reads and
writes. RenZip does not decompile `.rpyc` bytecode, and extracting an archive
does not give you the right to redistribute what is inside it.
