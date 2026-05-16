# CLI Architecture

## Binary
- `crates/rexx-cli`.

## Main Flags
- `--format`
- `--fix`
- `--output text|json|sarif`
- `--profile <name>` (default: `mainframe-compatible`)

## Behavior
- Formatting path uses formatter + selected profile.
- Linting path uses analyzer/rules only (no CLI heuristics).
