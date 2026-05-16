# Legacy Platform Support Strategy

## Objective
`rexxlint` keeps Rust as the canonical implementation for modern systems while adding a C99 portability layer for rare and legacy platforms historically seen in Regina-Rexx deployments.

This design avoids degrading Tier 1 architecture quality and developer velocity.

## Platform Tiers

### Tier 1: Primary
- Linux
- macOS
- Windows
- FreeBSD

Implementation:
- Full Rust engine (`crates/*`)
- Full lint pipeline
- Formatter
- JSON/SARIF
- Future LSP and IDE integrations

Current release-pipeline status (as of 2026-05-16):
- Linux x86_64
- macOS x86_64
- macOS aarch64
- Windows x86_64

### Tier 2: Secondary
- OpenBSD
- NetBSD
- illumos
- Solaris-like systems
- AIX

Implementation:
- Rust source-build support
- Best-effort CI build and test coverage
- Feature parity goal with Tier 1 where toolchains permit

### Tier 3: Legacy/Rare
- OS/2
- DOS
- Amiga
- AROS
- OpenVMS
- QNX
- BeOS/Haiku
- OpenEdition
- SkyOS
- Older UNIX variants where toolchains/libc allow

Implementation:
- `portable-c/` C99 fallback binary
- Minimal CLI (`rexxlint file.rexx`, `rexxlint --fix file.rexx`)
- Plain text output; optional JSON if target libc/toolchain supports cleanly
- No LSP/IDE/runtime-heavy features

## Repository Layout

```text
rexxlint/
  crates/
  portable-c/
    include/
    src/
      lexer.c
      parser_lite.c
      formatter_lite.c
      rules.c
      diagnostics.c
      main.c
    tests/
    Makefile
  docs/
  tests/
```

## Fallback Feature Scope (Tier 3)
Implemented in portable mode:
- R001 missing first-line comment
- R002 unclosed block comments
- R003 unmatched DO/END
- R004 unmatched SELECT/END
- trailing whitespace detection (`R009` in portable layer)
- basic formatting validation
- optional `--fix` behavior (minimal and deterministic)

Planned next in portable mode:
- R005 duplicate labels

Not in scope for portable mode:
- Full AST
- Advanced semantic analysis
- LSP server
- IntelliJ integration
- Advanced formatter normalization and style transforms

## Rule Parity Matrix (Rust Canonical vs C Fallback)

| Rule | Rust | portable-c | Notes |
|---|---|---|---|
| R001 | yes | yes | aligned |
| R002 | yes | yes | aligned |
| R003 | yes | yes | aligned |
| R004 | yes | yes | aligned |
| R005 | yes | planned | C backlog |
| R006 | yes | no | Rust-only for now |
| R007 | yes | no | Rust-only for now |
| R008 | yes | no | Rust-only for now |
| R009 | yes | yes | semantics differ: line-length (Rust) vs trailing-whitespace (C) |
| R010 | yes | no | Rust-only for now |

## Single Source of Truth for Rules
Rust remains canonical for rule semantics and diagnostics shape.

To keep C and Rust aligned:
1. Rule IDs are stable (`R001`..)
2. Severity mapping is consistent (`error`, `warning`)
3. Diagnostic fields are compatible: `rule_id`, `line`, `column`, `message`, `severity`
4. Golden test corpus under shared test data (next step)
5. C layer is subset-only and must never redefine meaning of already-shared rule IDs

## Portability Guidelines
- Strict ANSI C99 (`-std=c99`)
- No compiler-specific extensions unless fully guarded
- No threads required
- No dynamic loading
- No network access
- No external dependencies
- No non-standard filesystem APIs by default
- Prefer portable libc (`fopen`, `fread`, `snprintf`, `malloc`, `free`)
- Deterministic output ordering and message text

## Compiler Compatibility Matrix

| Platform class | Compiler baseline | Status target | Notes |
|---|---|---|---|
| Tier 1 | Rust stable + modern C compiler | Required | Rust canonical |
| Tier 2 | Rust stable where available | Best effort | some targets can be build-only CI |
| Tier 3 legacy POSIX-like | GCC 4.8+ / Clang 3.4+ (C99) | Required for fallback | avoid C11-only APIs |
| Very old vendor UNIX | vendor cc with C99 mode | Opportunistic | prioritize portability over strict warnings |
| DOS/OS2 variants | platform-specific GCC ports | Opportunistic | keep memory and I/O simple |

## CI Strategy

### Rust CI (current)
- `fmt`, `clippy`, `test`, `build`
- release artifacts for Tier 1 host-native matrix

### Portable C CI (current)
- dedicated job executes `make portable-all`

### Tier 2 CI rollout (recommended)
1. Add `cargo check --target` best-effort matrix for:
   - `x86_64-unknown-openbsd`
   - `x86_64-unknown-netbsd`
   - `x86_64-unknown-illumos`
2. Mark Tier 2 jobs non-blocking initially.
3. Promote stable targets to required checks after 4+ green weeks.

### Release Strategy
- Tier 1: publish Rust binaries
- Tier 3: publish source tarball for `portable-c/` and build instructions

## Migration Strategy Between Rust and C
1. Rust rules evolve first (canonical).
2. Rules explicitly marked `portable` are implemented in `portable-c/src/rules.c`.
3. Add/update shared golden samples for overlapping rules.
4. Validate deterministic diagnostics across both implementations for overlapping rules.
5. Document any known divergence explicitly in this file.

## Build Instructions (Portable C)

```bash
cd portable-c
make
./rexxlint-portable tests/sample.rexx
make test
```

## Testing Strategy
- Keep Rust snapshot tests and rule tests as primary semantic verification.
- Add/maintain portable-c smoke + integration tests for subset behavior.
- Add shared cross-runtime golden fixtures for overlapping rules.
- Ensure deterministic outputs for both runtimes on fixed inputs.

## Known Constraints
- Portable fallback is compatibility-focused, not feature parity-focused.
- Some legacy targets may require local toolchain patching or disabled warnings.
- JSON output may be disabled on the oldest environments if `snprintf`/buffer behavior is unreliable.
