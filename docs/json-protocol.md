# JSON Protocol

This document describes the JSON output format emitted by
`rexxlint check --output json` and the stability guarantees attached to it.

## Schema version

All JSON responses carry a top-level `"schema_version"` integer.

| Value | Meaning |
|---|---|
| `1` | current stable schema (this document) |

A breaking change to the schema increments this integer. Consumers should
reject or warn on unexpected values rather than silently misparse.

## Diagnostics schema (`check --output json`)

```jsonc
{
  "schema_version": 1,          // integer — protocol version
  "files": [                    // array — one entry per file that was linted
    {
      "file": "/abs/path/to/script.rexx",  // string — absolute path
      "diagnostics": [          // array — empty when the file is clean
        {
          "code":     "R001",   // string — rule identifier (e.g. "R001")
          "severity": "error",  // "error" | "warning" | "info"
          "message":  "Missing required first-line Rexx comment",
          "span": {
            "start_line": 1,    // 1-based
            "start_col":  1,    // 1-based
            "end_line":   1,
            "end_col":    1
          }
        }
      ]
    }
  ]
}
```

### Field notes

- `files` is always an array, even when a single path is checked.
- `files` is sorted lexicographically by `file` path — output is deterministic.
- A clean file produces `"diagnostics": []`, not the absence of the key.
- `span` columns are byte-indexed UTF-8 offsets, 1-based.
- `code` values match the rule identifiers in [`docs/rules/`](rules/).
- When invoked with `--stdin`, `files[0].file` is `"<stdin>"` by default.
  Pass `--path <virtual-path>` to override it with the real file name
  (recommended for editor integrations so gutter markers resolve correctly).

## Formatter schema (`format --output json`)

> Not yet implemented. The formatter currently operates silently (in-place,
> `--check`, or `--diff` modes). A JSON report mode is planned for a future
> minor release.

## SARIF output (`check --output sarif`)

See [`sarif.md`](sarif.md) for the full SARIF envelope structure. SARIF output
uses a `"$schema"` URI (per the SARIF 2.1.0 spec) instead of
`"schema_version"` — the SARIF specification forbids additional top-level
properties, so `schema_version` does **not** appear in SARIF output.

## Versioning policy

### Backward-compatible changes (no version bump)

- Adding optional fields to any object.
- Adding new entries to the `files` array for previously ignored paths.
- Adding new `code` values (new rules).

### Breaking changes (increments `schema_version`)

- Removing or renaming existing fields.
- Changing the type of an existing field.
- Changing the semantics of `severity` values.
- Reordering array entries in a way that breaks position-based parsing
  (current guarantee: `files` is always sorted by path).

## Consuming the JSON output

```bash
# All error-severity diagnostics across a directory
rexxlint check --output json src/ | \
  jq '[.files[].diagnostics[] | select(.severity == "error")]'

# File paths that have at least one diagnostic
rexxlint check --output json src/ | \
  jq -r '.files[] | select(.diagnostics | length > 0) | .file'

# Count total warnings
rexxlint check --output json src/ | \
  jq '[.files[].diagnostics[] | select(.severity == "warning")] | length'
```

## Stability commitment

- `schema_version: 1` fields documented here are **stable** as of v0.2.0-alpha.
- Fields not listed here should be treated as **unstable** and may change
  without a version bump during the alpha phase.
- Once v1.0.0 is released, all changes to `schema_version: 1` fields will
  require a minor or major version bump per semantic versioning.
