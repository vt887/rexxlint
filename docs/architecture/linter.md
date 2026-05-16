# Linter Architecture

## Current State
- Canonical lint execution lives in `crates/rexx-rules`.
- `rexx-analyzer` is orchestration-only pass-through.

## Rules
- Implemented: R001..R010.
- Rule results sorted by `(line, column, code)` for stable output.

## Cross-Implementation Strategy
- Rust rules are source of truth.
- Portable C subset mirrors rule IDs where implemented.
