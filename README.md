# rexxlint

Cross-platform Rexx linter/formatter with deterministic diagnostics and CI-friendly output.

## Current MVP

- `R001` missing first-line comment
- `R002` unclosed block comment
- `R003` unmatched `DO/END`
- `R004` unmatched `SELECT/END`
- `R007` unsafe `INTERPRET`
- Multi-file / directory scanning with `.gitignore` support
- `check` and `format` subcommands
- `--output text|json|sarif`

## Usage

```bash
# Lint one file, a directory, or multiple paths
cargo run -p rexx-cli -- check path/to/file.rexx
cargo run -p rexx-cli -- check src/ --output json
cargo run -p rexx-cli -- check src/ --output sarif

# Format — in-place rewrite
cargo run -p rexx-cli -- format src/

# Format — dry-run modes (never touch disk)
cargo run -p rexx-cli -- format --check src/   # exit 1 if any file would change
cargo run -p rexx-cli -- format --diff   src/  # print unified diff and exit 1

# Stdin mode (editor integrations — no temp files, no disk writes)
cat file.rexx | cargo run -p rexx-cli -- check --stdin --output json
cat file.rexx | cargo run -p rexx-cli -- format --stdin --path file.rexx

# Global options
cargo run -p rexx-cli -- --no-ignore check src/ # ignore .gitignore rules
cargo run -p rexx-cli -- --jobs 1    check src/ # disable parallelism
```

## Validate

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release -p rexx-cli
```

## Legacy Platforms

`rexxlint` includes a C99 compatibility layer for rare and legacy targets in `portable-c/`.
See [legacy-platform-support.md](docs/legacy-platform-support.md) for tiering, scope, and build constraints.
