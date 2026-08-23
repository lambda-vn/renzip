# Commit Convention

This project enforces **Conventional Commits**. Every commit message is validated
by a local pre-commit hook at commit time. A non-conforming commit will be
**rejected** before it is written to the repository.

---

## Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

All three parts of the first line are mandatory except the scope, which is optional
but strongly recommended. The first line must not exceed **100 characters**.

---

## Types

| Type       | When to use                                                               |
|------------|---------------------------------------------------------------------------|
| `feat`     | A new feature visible to the user or to the application's public API     |
| `fix`      | A bug fix                                                                 |
| `docs`     | Documentation only (markdown files, docstrings)                          |
| `refactor` | Code change that neither adds a feature nor fixes a bug                   |
| `test`     | Adding or updating tests, no production code change                       |
| `chore`    | Maintenance tasks: dependency updates, config files, CI tweaks            |
| `perf`     | A change that improves performance without altering behavior              |
| `ci`       | Changes to GitHub Actions workflows or CI configuration                   |

**Rules:**
- Use exactly one type per commit.
- Never mix a feature and a bug fix in the same commit. Split them.
- `refactor` must not change observable behavior. If it does, use `fix` or `feat`.

---

## Scope

The scope identifies the area of the codebase affected. It is written in lowercase,
using hyphens as separators.

| Scope        | Area                                                                        |
|--------------|-----------------------------------------------------------------------------|
| `header`     | Header line detection and parsing (`crates/renzip-core/src/header.rs`)      |
| `index`      | zlib and pickle decoding of the index (`crates/renzip-core/src/index.rs`)   |
| `archive`    | Opening, listing, reading, extracting (`crates/renzip-core/src/archive.rs`) |
| `builder`    | Scanning a directory and writing an archive (`.../src/builder.rs`)          |
| `path`       | Entry path sanitizing and containment (`crates/renzip-core/src/path.rs`)    |
| `ipc`        | Tauri commands and progress events (`src-tauri/src/lib.rs`)                 |
| `desktop`    | Handing a file to the desktop, per platform (`src-tauri/src/desktop.rs`)    |
| `preview`    | Deciding what to show for an entry (`src-tauri/src/preview.rs`)             |
| `state`      | Open archives and their scratch directories (`src-tauri/src/state.rs`)      |
| `tree`       | The entry tree: folding, ticking, search (`src/lib/tree.ts`, `EntryTree`)   |
| `browse`     | Browse view (`src/lib/views/BrowseView.svelte`)                             |
| `create`     | Create view (`src/lib/views/CreateView.svelte`)                             |
| `settings`   | Settings view and persisted preferences                                     |
| `i18n`       | UI translations (`src/lib/i18n.svelte.ts`)                                  |
| `ui`         | Cross-cutting UI: tokens, theme, shared components, virtual list            |
| `fixtures`   | Test archives and the script that writes them (`scripts/make_fixtures.py`)  |
| `ci`         | GitHub Actions workflows                                                    |
| `deps`       | Cargo or npm dependency changes                                             |

The scope is **optional** but should be included whenever the change is confined
to a clearly identifiable area. Omit it only for cross-cutting changes that affect
multiple areas simultaneously.

---

## Description

- Written in **English**, imperative mood, present tense.
- **Do not** capitalize the first letter.
- **Do not** end with a period.
- Describe *what* the commit does, not *why* (the why goes in the body).

```
# Correct
feat(tree): add a context menu on the entry rows
fix(path): reject components ending with a dot or a space
docs(security): document the containment check on extraction

# Wrong
feat(tree): Added a context menu on the entry rows.    <- past tense + period
fix: fixed a bug                                       <- vague, no scope
feat(tree): Context menu                               <- not imperative mood
```

---

## Body

The body is optional but expected for any non-trivial commit. It explains **why**
the change was made, and any context that is not obvious from the diff.

- Separated from the description by a blank line.
- Wrap lines at **72 characters**.
- May include multiple paragraphs.

```
fix(index): read an empty prefix written by protocol 2

Python pickles b"" as a call to __builtin__.bytes(), which
serde-pickle reports as an unresolved global rather than as a value.
Every archive written by Ren'Py itself failed to open on that alone.
The deserializer now resolves those globals, and a None prefix is
read as an empty byte string, which is what the engine does.
```

---

## Footer

Used for three purposes only:

**Breaking changes:**
```
feat(ipc): return an outcome from open_entry

BREAKING CHANGE: open_entry returned nothing and now returns an
OpenOutcome. Update the caller in src/lib/api.ts accordingly.
```

**Issue references:**
```
fix(path): prevent path traversal in archive entry names

Closes #42
```

**AI attribution (mandatory when an LLM contributed):**

When a commit contains code or a message produced with the help of an LLM,
its use must be disclosed, in one of two equivalent ways:

- The LLM adds a `Co-Authored-By` trailer identifying itself, one per line,
  as the last lines of the message:
  ```
  feat(parser): add support for multiline dialogue blocks

  Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
  ```
- Or the human author states the LLM use explicitly in the body:
  ```
  feat(parser): add support for multiline dialogue blocks

  Drafted with the help of an LLM, reviewed and validated by the author.
  ```

A commit that involved no LLM carries no such trailer or note.

---

## Full examples

**Simple feature, no body needed:**
```
feat(settings): add the interface language to the general section
```

**Bug fix with context:**
```
fix(archive): check containment on the canonical path

The check ran on the entry path as it was read, which a symbolic link
already sitting in the destination directory walks straight past. It
now resolves the parent directory before writing and refuses the entry
when the result falls outside the destination, which is the only form
of the check the file system cannot lie about.
```

**Dependency update:**
```
chore(deps): update tauri to 2.11.5
```

**Format support:**
```
feat(header): read the RPA-3.2 header line

3.2 carries an extra field between the offset and the key, so the fold
that builds the key starts one word further than in 3.0. Reading it
costs one match arm; writing it is still refused, since no reference
implementation says which field that is.
```

**Test addition:**
```
test(fixtures): add an archive whose entries carry a prefix
```

**Documentation:**
```
docs: add COMMIT_CONVENTION.md
```

---

## What the commit-msg hook rejects

Commit messages are validated by a local pre-commit hook running
`scripts/check_commit_msg.py` at the `commit-msg` stage. It matches the first
line against this pattern:

```
^(feat|fix|docs|style|refactor|test|chore|ci|perf|build|revert)(\([a-zA-Z0-9_\-\.]+\))?(!)?: .+
```

Any commit whose first line does not match is rejected immediately. Common
mistakes that trigger a rejection:

- Using a type outside the pattern (e.g. `update`, `wip`)
- Forgetting the colon and space after the type or scope
- Using uppercase for the type (e.g. `Feat` instead of `feat`)
- An empty description after the colon

The hook only checks the first line's shape. The 100-character limit, the
lowercase-hyphenated scope, and the curated type set documented above are
project conventions, not enforced by the hook: `style`, `build`, and `revert`
match the pattern but are not used in this project. The optional `!` marks a
breaking change (`feat!:`).

---

## Guidance for AI agents

When generating a commit message:

1. Read the diff carefully before writing the message.
2. Identify a single coherent intent. If the diff contains multiple unrelated
   changes, request that they be split into separate commits rather than combining
   them under a vague description.
3. Choose the most specific scope available in the table above. If the change
   spans two scopes equally, omit the scope rather than picking an imprecise one.
4. The description must describe the observable effect of the change, not the
   implementation detail. Prefer `add support for X` over `update regex in Y`.
5. Always write the body when the change involves a security decision, a
   non-obvious workaround, or a constraint imposed by an external system
   (the RPA format, a Tauri limitation, a Windows path rule, etc.).
6. Never invent a type that is not in the allowed list.
7. Never use `WIP` or similar markers. A commit must represent a complete,
   reviewable unit of work.
8. Always add a `Co-Authored-By` trailer identifying yourself as the last
   line of any commit you generate (see the AI attribution footer). Never
   remove an existing one.
