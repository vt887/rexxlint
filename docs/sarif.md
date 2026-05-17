# SARIF Output

rexxlint supports the [Static Analysis Results Interchange Format (SARIF) 2.1.0](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html).

## Generating SARIF

```bash
rexxlint check --output sarif src/ > results.sarif
```

Exit codes: 0 = no findings, 1 = findings present, 2 = hard error.

## Output structure

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",         // SARIF spec version
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "rexxlint",
          "rules": [
            {
              "id": "R001",
              "shortDescription": { "text": "Missing first-line comment" },
              "help": { "text": "Rule R001: Missing first-line comment" },
              "defaultConfiguration": { "level": "error" }
            }
            // ... additional rules
          ]
        }
      },
      "results": [
        {
          "ruleId": "R001",
          "level": "error",
          "message": { "text": "Missing required first-line Rexx comment" },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": { "uri": "src/main.rexx", "uriBaseId": "%SRCROOT%" },
                "region": {
                  "startLine": 1,
                  "startColumn": 1,
                  "endLine": 1,
                  "endColumn": 1
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

## `$schema` field

The top-level `"$schema"` field points to the SARIF 2.1.0 JSON Schema and
allows validators to confirm the document structure. It is always present in
rexxlint SARIF output.

The rexxlint-specific protocol version is tracked by `"schema_version"` in
the **JSON output format** (`--output json`) only — it does not appear in
SARIF, which forbids additional top-level properties per the spec.

## Severity mapping

| rexxlint severity | SARIF level |
|---|---|
| `error` | `error` |
| `warning` | `warning` |
| `info` | `note` |

## GitHub Code Scanning integration

Upload SARIF to GitHub Code Scanning so findings appear in the Security tab
and as PR annotations.

### In your own repository

```yaml
# .github/workflows/rexxlint.yml
name: rexxlint

on: [push, pull_request]

permissions:
  contents: read
  security-events: write

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Install rexxlint
        run: |
          curl -LO https://github.com/vt887/rexxlint/releases/latest/download/rexxlint-linux-x86_64.tar.gz
          tar -xzf rexxlint-linux-x86_64.tar.gz rexxlint
          chmod +x rexxlint

      - name: Run rexxlint
        run: ./rexxlint check --output sarif . > results.sarif
        continue-on-error: true

      - uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: results.sarif
          category: rexxlint
```

A complete example is available in [`examples/github-actions/code-scanning.yml`](../examples/github-actions/code-scanning.yml).

## Validating SARIF locally

```bash
# Minimal structural check using jq
jq -e '
  .version == "2.1.0" and
  ((.runs | length) > 0) and
  (.runs[0].tool.driver.name | type) == "string" and
  (.runs[0].results | type) == "array"
' results.sarif && echo "SARIF is valid"
```

For full schema validation, use the
[Microsoft SARIF validator](https://sarifweb.azurewebsites.net/) or
[`sarif-tools`](https://pypi.org/project/sarif-tools/) (`pip install sarif-tools`).
