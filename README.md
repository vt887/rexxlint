# rexxlint

Cross-platform Rexx linter/formatter with deterministic diagnostics and CI-friendly output.

## Current MVP

- `R001` missing first-line comment
- `R002` unclosed block comment
- `R003` unmatched `DO/END`
- `R004` unmatched `SELECT/END`
- `R007` unsafe `INTERPRET`
- `--format`, `--fix`
- `--output text|json|sarif`

## Usage

```bash
cargo run -p rexx-cli -- path/to/file.rexx
cargo run -p rexx-cli -- --format path/to/file.rexx
cargo run -p rexx-cli -- --fix --output json path/to/file.rexx
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
See [legacy-platform-support.md](/Users/tymoshv/MyPetProjects/rexxlint/docs/legacy-platform-support.md) for tiering, scope, and build constraints.
