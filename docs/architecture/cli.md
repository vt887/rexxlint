# CLI Architecture

## Binary
- `crates/rexx-cli`

## Subcommands

```
rexxlint check  [PATHS...] [--output text|json|sarif] [--jobs N] [--no-ignore]
rexxlint format [PATHS...] [--check] [--diff] [--profile NAME] [--jobs N] [--no-ignore]
```

`PATHS` may be files or directories; directories are traversed recursively.
Bare `rexxlint` (no args) prints help and exits 0.

## Global flags
- `--jobs N` — parallelism degree; default = available CPUs; `1` = sequential
- `--no-ignore` — disable `.gitignore` / `.ignore` file filtering

## `check` behaviour
- Discovers all `.rexx`/`.rex`/`.rx` files under each path via `rexx-walker`
- Lints every file in parallel (rayon), results sorted by path
- Outputs diagnostics in the chosen format (text default, json, sarif)
- Exit 0 = no diagnostics; exit 1 = one or more diagnostics; exit 2 = IO/parse error

## `format` behaviour
- **Default (in-place)**: writes formatted output atomically via `tempfile`; preserves Unix mode bits
- **`--check`**: never touches disk; exits 1 and prints `would reformat: <path>` per changed file
- **`--diff`**: never touches disk; prints unified diff per changed file; exits 1 if any diff
- `--check --diff` does both
- Formatter failure on one file is reported but does not abort the remaining files

## Output schema
JSON and SARIF payloads include `"schema_version": 1`.

## Pipeline
```
parse args → resolve_config(cwd) → rexx-walker::discover(paths)
  → rayon::par_iter → sort by path → render output → exit code
```
