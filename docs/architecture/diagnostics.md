# Diagnostics Architecture

## Shared Model
- `Diagnostic { code, severity, message, line, column }`
- Severity enum: `error | warning`

## Output Targets
- text
- JSON
- SARIF 2.1.0

## Compatibility
- Rule IDs and severity semantics are aligned with portable C strategy.
