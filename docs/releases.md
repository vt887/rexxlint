# Releases

## Versioning

rexxlint follows [Semantic Versioning](https://semver.org/).

- **Patch** (`x.y.Z`): bug fixes, no behaviour change.
- **Minor** (`x.Y.0`): new rules or CLI flags, backward-compatible.
- **Major** (`X.0.0`): breaking changes to CLI contract or JSON/SARIF schema.

The single source of truth for the version is `[workspace.package] version`
in the root `Cargo.toml`. The build date is stamped at compile time by
`crates/rexx-cli/build.rs` and exposed in the CLI banner.

## Release process

1. Bump `version` in `Cargo.toml` and run `cargo build` to update `Cargo.lock`.
2. Commit: `git commit -am "chore: release vX.Y.Z"`.
3. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. The `release.yml` workflow triggers automatically, builds all five
   platform targets, and creates a GitHub Release with binaries + checksums.

## Release artifacts

| File | Platform |
|---|---|
| `rexxlint-linux-x86_64.tar.gz` | Linux x86-64 |
| `rexxlint-linux-aarch64.tar.gz` | Linux AArch64 (ARM64) |
| `rexxlint-macos-x86_64.tar.gz` | macOS x86-64 (Intel) |
| `rexxlint-macos-aarch64.tar.gz` | macOS AArch64 (Apple Silicon) |
| `rexxlint-windows-x86_64.zip` | Windows x86-64 |
| `SHA256SUMS` | SHA-256 checksums for all archives |

Each archive contains:

```
rexxlint (or rexxlint.exe)
LICENSE
README.md
```

## Verifying a release

```bash
# Download the archive and checksum file
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-linux-x86_64.tar.gz
curl -LO https://github.com/vt887/rexxlint/releases/latest/download/SHA256SUMS

# Verify
sha256sum --check --ignore-missing SHA256SUMS
```

## Pre-release / manual builds

The `release.yml` workflow can be triggered manually via `workflow_dispatch`
with an optional `tag` input. This is useful for testing the release pipeline
before cutting an official tag.

## CI-only artifacts

The `ci.yml` workflow uploads a `rexxlint-linux-x86_64-ci` artifact on every
successful CI run. These are kept for 7 days and are intended for internal
testing only — they are not official releases.
