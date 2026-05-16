# Phase 1 Plan — rexxlint

## Goal

Deliver a working, publishable Rust workspace that can lint and format Rexx
source files on any tier-1 platform (Linux x86-64, macOS arm64/x86-64,
Windows x64) with a portable C99 fallback for legacy systems.

## Scope

### In scope

- Cargo workspace with all crates scaffolded and compiling
- Lint rules R001–R010 with unit tests
- Text, JSON (with file path), and SARIF output formats
- Formatting profiles: `mainframe-compatible`, `standard`, `minimal`
- Basic source formatter (keyword normalisation, indentation, trailing whitespace)
- Portable C99 layer scaffold (`portable-c/`) — compilable, basic rules
- CI pipeline: `cargo fmt`, `cargo clippy`, `cargo test`, C99 build
- Docs: architecture overview, rule pages R001–R010, legacy-platform-support

### Out of scope for Phase 1

- LSP / editor integration (Phase 2)
- IntelliJ plugin (Phase 3)
- `--fix` auto-rewrite (Phase 2)
- Full AST-level analysis (parser is scaffold only in Phase 1)
- Rule `--rule` filter flag
- Mixed-indentation detection beyond tab-presence check
- Inline `DO` after `THEN/ELSE` block tracking

## Deliverables

| # | Deliverable | Status |
|---|---|---|
| 1 | Cargo workspace compiles clean | ✅ Done |
| 2 | `rexx-diagnostics` shared types | ✅ Done |
| 3 | `rexx-lexer` scaffold | ✅ Done |
| 4 | `rexx-parser` scaffold | ✅ Done |
| 5 | `rexx-rules` R001–R010 with tests | ✅ Done |
| 6 | `rexx-formatter` basic pass | ✅ Done |
| 7 | `rexx-config` three profiles | ✅ Done |
| 8 | `rexx-cli` binary (text/JSON/SARIF, exit codes) | ✅ Done |
| 9 | `portable-c/` scaffold builds with Make | ✅ Done |
| 10 | CI (`cargo fmt` + `clippy` + `test` + C build) | ✅ Done |
| 11 | `docs/architecture.md` + `docs/rules/R001–R010.md` | ✅ Done |
| 12 | `docs/legacy-platform-support.md` | ✅ Done |
| 13 | `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, `LEAN-CTX.md` | ✅ Done |
| 14 | `rust-toolchain.toml` pinned to stable | ✅ Done |
| 15 | `README.md` with goals and roadmap | ✅ Done |

## Known Limitations (tracked for Phase 2)

1. **Formatter indentation** — nesting depth not tracked; all body lines get one fixed indent level
2. **Inline DO** — `IF x THEN DO` not detected by R003/R004 block-balance check
3. **`--rule` filter** — not yet implemented in CLI
4. **`--format` diff mode** — emits full formatted source, not a unified diff
5. **JSON file path** — now included (`{"file": ..., "diagnostics": [...]}`); schema may expand
6. **Portable-C string masking** — `/* */` inside string literals affects comment tracker
7. **R009/R010 Rust vs C divergence** — C layer will remap trailing-whitespace to R011 in Phase 2

## Phase 2 Preview

- Full recursive-descent parser producing typed AST
- Rules R011–R020 (data-flow, uninitialized variables, SAY-without-quotes)
- LSP server (`rexx-lsp`) with `textDocument/publishDiagnostics`
- `--fix` auto-rewrite for mechanical rules
- `--rule` filter flag
- Depth-tracked formatter indentation

## Timeline

Phase 1 is considered complete when all CI checks pass on the `main` branch and
PR #1 is merged.
