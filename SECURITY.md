# Security Policy

## Supported versions

| Version | Supported |
|---|---|
| `0.2.x-alpha` (current) | ✅ security fixes |
| `< 0.2` | ❌ no longer supported |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Report vulnerabilities privately via **GitHub Private Vulnerability Reporting**:

1. Go to the [Security tab](https://github.com/vt887/rexxlint/security/advisories/new).
2. Click **"Report a vulnerability"**.
3. Fill in the details: affected versions, reproduction steps, potential impact.

If Private Vulnerability Reporting is unavailable, email the maintainer directly
(address on the GitHub profile) with the subject line `[rexxlint] Security vulnerability`.

## Response timeline

| Stage | Target |
|---|---|
| Acknowledgement | ≤ 48 hours |
| Initial assessment | ≤ 5 business days |
| Patch or mitigation | ≤ 30 days for critical, 90 days for moderate |

## Scope

In scope:
- Arbitrary code execution via malformed Rexx input
- Path traversal via file discovery (`rexx-walker`)
- Output injection in JSON/SARIF that could affect downstream consumers
- Credential or secret exposure via CLI output

Out of scope:
- Denial-of-service on intentionally malformed input (linters parse untrusted code by design)
- Vulnerabilities in optional dev-only dependencies (`criterion`, `insta`)

## Disclosure policy

We follow **coordinated disclosure**. We ask reporters to allow reasonable
remediation time before public disclosure. We credit reporters in the release
notes unless they prefer anonymity.
