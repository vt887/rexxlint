# CI/CD Architecture

## Current Workflows
- `.github/workflows/ci.yml`
  - trigger: push/pull_request (`main|master|develop`) + manual
  - jobs:
    - `quality`: `make fmt-check`, `make lint`, `make test`, `make build`
    - `portable`: `make portable-all`
    - `package`: `make release` + artifact packaging/upload
  - concurrency cancellation enabled per ref
  - permission scope: `contents: read`
- `.github/workflows/release.yml`
  - trigger: tag `v*` + manual
  - matrix build on host-native runners:
    - linux x86_64
    - macOS x86_64
    - macOS aarch64
    - windows x86_64
  - publish job aggregates artifacts and creates GitHub Release
  - permission scope: `contents: write`

## Notes
- Pipelines use `Makefile` targets as single source of execution commands.
- Linux aarch64 artifact is currently not part of release matrix to avoid unstable default-runner cross-linking.
