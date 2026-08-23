# Security Policy

## Supported versions

The project is in beta. Only the latest commit on `main` receives fixes;
there is no maintained release branch yet.

## Reporting a vulnerability

Please report privately through GitHub, using **Security > Report a
vulnerability** on this repository. Do not open a public issue for a
vulnerability.

Include what you did, what happened, and what you expected. The header
line of the archive and the entry path involved usually say more than a
screenshot. Do not attach the archive itself unless it is one you built
for the report: a game archive holds assets you are very unlikely to be
allowed to redistribute, and a crafted 300 byte reproducer is worth more
anyway. `scripts/make_fixtures.py` writes exactly that kind of file.

Expect a first answer within a week. The project is maintained by one
person on their own time, so a fix may take longer than the
acknowledgement.

## The threat model in one line

**A `.rpa` file is untrusted input.** It was downloaded from somewhere,
it was produced by a tool nobody here controls, and every byte in it,
including the paths and the lengths, is attacker-chosen. Everything
below follows from that.

## What is in scope

- **The index.** It is a Python pickle, and unpickling untrusted data in
  Python is arbitrary code execution. That is a large part of why the
  reader is written in Rust: `serde-pickle` parses the opcodes into
  values and never constructs an object a stream asks for. On top of
  that, `crates/renzip-core/src/index.rs` accepts one shape only, a map
  of strings to lists of two or three element tuples, and treats
  anything else as a malformed archive rather than a case to be clever
  about.

- **Entry paths, which is where extractors get broken.** `path.rs`
  refuses absolute paths, drive letters, `..` components, Windows
  reserved device names (`CON`, `NUL`, `COM1`...), the characters no
  Windows path may hold, and components ending in a dot or a space,
  which Windows silently strips. Then, and this is the check that
  actually counts, `archive.rs` canonicalises the parent directory
  before writing and refuses the entry if the result is not contained in
  the destination directory. A refused entry does not abort the
  extraction: it is listed in the report and skipped, so one hostile
  path in an archive of forty thousand does not cost you the other
  thirty-nine thousand nine hundred and ninety-nine.

- **Symbolic links already on disk.** An archive cannot create one, but
  it can name a path where one already sits, which would let a write
  land anywhere the link points. Both the parent directory and the
  target file are checked, and an existing link at the target is
  removed rather than written through.

- **Declared offsets and lengths.** They are read from the index, so
  they are attacker-chosen too, and a plausible index can ask for
  bytes past the end of the file or announce a prefix longer than the
  entry that holds it. Both are validated against the real file size
  when the archive is opened, before anything is allocated or read, and
  a failing entry is rejected with the numbers it declared. Nothing
  loads an archive whole: reads are `seek` plus a bounded `take`.

- **Preview.** Text stops at 1 MiB and images at 8 MiB, decided on the
  declared size before the bytes are read. Images are handed to the
  webview as a `data:` URL under a CSP that allows `data:` for images
  only; a file whose bytes are not what its extension claims is a
  broken image in a panel, not a decoder running on something else.

- **Handing a file to the desktop.** The row menu writes a copy into a
  scratch directory and asks the system to open it. This is the one
  place where the content of an untrusted archive reaches a program
  that is not this one, and it is worth being clear about: opening a
  `.png` from an unknown archive runs your image viewer on
  attacker-chosen bytes. RenZip does not decide which program that is,
  it asks the desktop, exactly as a double click in your file manager
  would. What it does control is the path: the copy goes to
  `<temp>/renzip/<pid>-<timestamp>/`, one directory per open archive, so
  two archives holding the same `script.rpyc` cannot overwrite each
  other's copy and you cannot be shown the wrong file. Those
  directories are removed when the archive is closed and when the
  application exits.

- **The window's own capabilities.** The webview is granted
  `core:default`, the file dialogs, and writing text to the clipboard.
  The opener plugin is deliberately absent from that list: there is no
  command reachable from the frontend that takes a path and opens it.
  The row menu goes through `open_entry`, which takes an archive handle
  and an entry name and builds the path itself.

## What is not in scope

The scratch directory under the system temporary folder is local user
data, not a trust boundary. A process able to read it runs under your
account and could read the archive directly.

RenZip does not decompile `.rpyc` bytecode and does not execute anything
it extracts. If you extract a `.rpy` and run the game, you are running
that game's code, which is a decision made outside this application.

Binaries produced by the build workflow are **not signed**. Windows
SmartScreen and macOS Gatekeeper will warn about them, and that warning
means unsigned, not tampered with. Verifying that a download matches
what the workflow built is on you until signing exists.
