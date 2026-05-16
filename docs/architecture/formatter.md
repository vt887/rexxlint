# Formatter Architecture

## Current State
- Entrypoint: `crates/rexx-formatter/src/lib.rs`.
- Conservative transformations only:
  - first-line Rexx comment insertion
  - trailing space removal
  - tab expansion
  - controlled indentation
  - uppercase keyword normalization (profile-driven)

## Profile Integration
- Uses `rexx-config` profiles.
- Default profile: `mainframe-compatible`.

## Determinism
- Formatting is pure function of `(input, profile)`.
- Strings/comments are not rewritten for keyword normalization.
