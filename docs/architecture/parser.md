# Parser Architecture

## Current State
- Parser is currently parser-lite behavior implemented through rule-context tokenization (`crates/rexx-rules/src/context.rs`).
- No full AST parser is active yet.

## Deterministic Guarantees
- String and comment masking before token extraction.
- Stable token order and column mapping.

## Future Plan
- Replace parser-lite with dedicated `rexx-parser` crate implementation.
- Keep rule interface stable (`Vec<Diagnostic>` output).
