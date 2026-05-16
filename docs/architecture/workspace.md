# Rust Workspace Architecture

## Crates
- `rexx-lexer`
- `rexx-parser`
- `rexx-ast`
- `rexx-analyzer`
- `rexx-rules`
- `rexx-formatter`
- `rexx-diagnostics`
- `rexx-config`
- `rexx-cli`
- `rexx-lsp` (future)

## Principle
- Narrow crate responsibilities.
- Shared diagnostics/config crates.
- CLI as integration boundary.
