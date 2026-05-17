# CI/CD

## Workflows

| File | Trigger | Purpose |
|---|---|---|
| `ci.yml` | push/PR to main, develop | Quality gate — format, lint, test, bench, SARIF validation, CI artifact |
| `release.yml` | tag `v*`, manual | Cross-platform release binaries + GitHub Release |
| `code-scanning.yml` | push/PR to main, weekly | SARIF upload to GitHub Code Scanning |

## CI jobs (`ci.yml`)

```
fmt            → cargo fmt --all -- --check
clippy         → cargo clippy --all-targets --all-features -- -D warnings
test           → cargo test --all
bench          → cargo bench --no-run          (non-blocking: continue-on-error)
portable       → make portable-all             (C99 layer)
sarif-validate → build CLI, lint sample file, validate SARIF envelope with jq
package        → cargo build --release, tar with LICENSE + README (7-day artifact)
```

`bench` uses `continue-on-error: true` — a benchmark compilation failure is
surfaced as a warning, not a blocked merge.

`sarif-validate` asserts:
- `version == "2.1.0"`
- `runs` is non-empty
- `runs[0].tool.driver.name` is a string
- `runs[0].results` is an array

## Release matrix (`release.yml`)

| Target | Runner | Notes |
|---|---|---|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | default host |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | cross-compiled via `gcc-aarch64-linux-gnu` |
| `x86_64-apple-darwin` | `macos-13` | Intel Mac |
| `aarch64-apple-darwin` | `macos-14` | Apple Silicon |
| `x86_64-pc-windows-msvc` | `windows-latest` | MSVC toolchain |

Each artifact includes the binary, `LICENSE`, and `README.md`.
A `SHA256SUMS` file is generated in the `publish` job and attached to the GitHub Release.

## Caching

All Rust jobs use `Swatinem/rust-cache@v2`, keyed per target triple for the
release matrix. This caches `~/.cargo/registry` and `target/` between runs,
cutting cold-build times by roughly 60–80 % after the first run.

## Permissions

| Workflow | Permission |
|---|---|
| `ci.yml` | `contents: read` |
| `release.yml` | `contents: write` (to create releases) |
| `code-scanning.yml` | `contents: read`, `security-events: write` |

No secrets are required beyond the default `GITHUB_TOKEN`.

## Concurrency

All workflows cancel in-progress runs for the same `ref` (except `release.yml`
where `cancel-in-progress: false` prevents a concurrent release from racing).

## Running CI locally

```bash
make fmt-check   # mirrors the fmt job
make lint        # mirrors the clippy job
make test        # mirrors the test job
make release     # mirrors the build step in package/release jobs
cargo bench --no-run  # mirrors the bench job
```
