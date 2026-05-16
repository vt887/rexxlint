# rexxlint — Architecture Overview

This file is an entry-point index. Each topic links to a dedicated page in
[`docs/architecture/`](architecture/).

## Workspace

The repository is a Cargo workspace under `crates/` with ten crates and a
portable C99 layer in `portable-c/`. See [workspace.md](architecture/workspace.md)
for the full dependency graph and design rationale.

## Crates

| Crate | Purpose |
|---|---|
| `rexx-diagnostics` | Shared `Diagnostic`, `Severity`, and rule-ID types used by every other crate |
| `rexx-lexer` | Tokeniser — converts raw Rexx source into a flat token stream |
| `rexx-parser` | Recursive-descent parser — produces an AST from the token stream |
| `rexx-ast` | AST node types shared between parser and rules |
| `rexx-rules` | Lint rules R001–R010; each rule is an independent module |
| `rexx-formatter` | Source formatter; reads AST/tokens and emits normalised source |
| `rexx-config` | Formatting profiles (`mainframe-compatible`, `standard`, `minimal`) |
| `rexx-analyzer` | Orchestrates rules over a file or workspace |
| `rexx-cli` | Binary entry-point; argument parsing, output rendering (text/JSON/SARIF) |
| `rexx-lsp` | Language Server Protocol adapter (planned Phase 2) |

## Portable C layer

`portable-c/` provides a C99 implementation of a subset of the linter for
platforms where Rust is not available (z/OS, AIX, HP-UX, older POWER). The C
layer targets the same rule IDs and output formats as the Rust layer so tooling
can consume either. See [legacy-platform-support.md](legacy-platform-support.md).

## Further reading

- [CLI design](architecture/cli.md)
- [Parser design](architecture/parser.md)
- [Linter / rules engine](architecture/linter.md)
- [Formatter](architecture/formatter.md)
- [Diagnostics model](architecture/diagnostics.md)
- [SARIF output](architecture/sarif.md)
- [Formatting profiles](architecture/profiles.md)
- [CI/CD](architecture/cicd.md)
- [IntelliJ plugin plan](architecture/intellij-plan.md)
