# SARIF Architecture

## Version
- SARIF `2.1.0`

## Included Fields
- rule id
- severity level
- message
- file path
- line/column region
- rule metadata in driver section

## Implementation
- `crates/rexx-cli/src/lib.rs` (`render_sarif`).
