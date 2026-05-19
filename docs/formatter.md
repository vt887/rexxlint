# Formatter

`rexxlint format` rewrites Rexx source files to a canonical style defined by the
active formatting profile. The formatter is built on the same AST that the linter
uses, so formatting is always syntactically aware.

## Guarantees

| Guarantee | Detail |
|-----------|--------|
| **Idempotent** | `format(format(x)) == format(x)` for every input |
| **Non-destructive** | Comments are preserved exactly; string literal *content* is preserved but delimiters are normalized to single quotes (`'`) |
| **Deterministic** | Same input always produces the same output |
| **Safe for legacy code** | Malformed input is formatted best-effort; files are never corrupted |
| **Atomic writes** | Files are written via a temp-file + rename; a crash mid-write leaves the original intact |

## Usage

```bash
# Format files in-place
rexxlint format src/

# Dry-run: report which files would change, exit 1 if any
rexxlint format --check src/

# Show a unified diff without writing
rexxlint format --diff src/

# Format and also apply safe lint auto-fixes
rexxlint format --fix src/

# Select a formatting profile
rexxlint format --profile=standard src/
```

## Profiles

| Profile | Indent | Keywords | Max blank lines | Tabs |
|---------|--------|----------|-----------------|------|
| `mainframe-compatible` (default) | 4 spaces | UPPER | 1 | forbidden |
| `standard` | 4 spaces | lower | 1 | allowed |
| `minimal` | 4 spaces | preserve | 2 | allowed |

Profiles can be overridden per-project in `rexxlint.toml`. When a `[formatting]`
section is present it takes precedence over the `--profile` flag:

```toml
[formatting]
indent_size = 4
keyword_casing = "upper"   # "upper" | "lower" | "preserve"
max_blank_lines = 1
insert_first_comment = true
tabs_forbidden = true
line_length_soft = 72
line_length_hard = 80
```

## What the formatter changes

- **Keyword casing** — keywords (`DO`, `END`, `IF`, `SAY`, …) are normalised to
  the casing defined by the profile.
- **Indentation** — statements inside `DO`/`END`, `SELECT`/`END`, and
  `IF`/`THEN`/`ELSE` blocks are indented by `indent_size` spaces per level.
- **Trailing whitespace** — trailing spaces and tabs are removed from every line.
- **Blank lines** — runs of consecutive blank lines are collapsed to at most
  `max_blank_lines`.
- **First-line comment** — when `insert_first_comment = true` and the file does
  not start with a comment, a standard Rexx header comment is prepended.
- **Trailing newline** — every file ends with exactly one newline.

## What the formatter does NOT change

- String literal contents (`'hello'`, `"world"`)
- Comment text
- Statement order or logic
- Continuation semantics (`/*` style or `,` continuations)
- Labels
- Program semantics of any kind

## Idempotency test suite

The corpus at `tests/corpus/formatter/` is run on every CI build:

```
tests/corpus/formatter/
  valid/      — well-formed files; must be idempotent
  malformed/  — edge cases; formatter must not panic, output must be idempotent
  legacy/     — mainframe-style code; string literals must survive intact
```

Run the corpus locally:

```bash
cargo test -p rexx-formatter --test corpus_idempotency
```
