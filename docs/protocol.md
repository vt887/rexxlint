# rexxlint Protocol Reference

This document covers the stdin/stdout integration protocol used by editors, IDEs,
and CI tooling. For the full JSON diagnostics schema see [json-protocol.md](json-protocol.md).

## Stdin mode

Both `check` and `format` accept `--stdin` to read source from standard input and
write results to standard output. No temporary files are created and no disk writes
occur.

### Flags

| Flag | Commands | Description |
|------|----------|-------------|
| `--stdin` | `check`, `format` | Read source from stdin |
| `--path <PATH>` | `check`, `format` | Virtual file path used in output (default: `<stdin>`) |
| `--output <MODE>` | `check` | Output format: `text` \| `json` \| `sarif` (default: `text`) |
| `--check` | `format` | Exit 1 if stdin would be reformatted; no stdout output |
| `--diff` | `format` | Print unified diff to stdout; exit 1 if stdin would change |
| `--profile <PROFILE>` | `format` | Formatting profile (default: `mainframe-compatible`) |

### Incompatibilities

- `--stdin` cannot be combined with positional file paths.
- `check --stdin --fix` / `--fix-only` are rejected (no file to write back).

## Real-time linting (editor → rexxlint → JSON)

```bash
rexxlint check --stdin --output json --path "$VIRTUAL_FILE_PATH"
```

- Write the current buffer content to the process's stdin.
- Parse the JSON response from stdout (schema: [json-protocol.md](json-protocol.md)).
- If stderr is non-empty or exit code is 2, treat as a fatal error.

## Formatting (editor → rexxlint → formatted source)

```bash
rexxlint format --stdin --path "$VIRTUAL_FILE_PATH"
```

- Write the current buffer content to stdin.
- Replace the buffer with the full stdout output.
- If stderr is non-empty or exit code is 2, do **not** replace the buffer.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success — no diagnostics / file is already formatted |
| `1` | Diagnostics found, or file would be reformatted |
| `2` | Fatal error (bad arguments, I/O error, internal crash) |

## Determinism

Given the same input and profile, stdout is byte-for-byte identical across runs.
Formatting is idempotent: `format(format(x)) == format(x)`.
