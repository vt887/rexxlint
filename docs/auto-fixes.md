# Auto-fixes

`rexxlint` can automatically repair a subset of diagnostics. Auto-fixes are
**safe**: they never change program behaviour, never reorder statements, and
never modify string or comment contents.

## Enabling auto-fixes

```bash
# Lint and apply fixes; report remaining diagnostics
rexxlint check --fix src/

# Apply fixes silently (no diagnostic output)
rexxlint check --fix-only src/

# Format and apply lint fixes in one pass
rexxlint format --fix src/
```

> **Note:** `--fix` and `--fix-only` cannot be combined with `--stdin` because
> there is no file to write back to.

## Fix safety model

A fix is considered **safe** if it satisfies all of the following:

1. The replacement text is derived mechanically from the original token (e.g. case
   conversion).
2. No tokens are reordered.
3. No tokens are removed.
4. String and comment contents are untouched.
5. The fix applies to a single contiguous byte range.

Fixes that do not meet these criteria are not implemented. An `--unsafe-fixes`
flag is planned for a future release to opt into riskier transformations; it is
not yet available in the CLI.

## Fix application engine

The engine in `rexx-formatter::fix_applicator`:

1. Resolves every `Fix` span to a byte range in the source string.
2. Sorts fixes by ascending byte offset — **earliest fix wins** in case of overlap.
3. Uses a greedy non-overlapping selection: once a region is claimed, later fixes
   that intersect it are silently dropped.
4. Applies accepted fixes in descending order so byte offsets are not shifted.

This guarantees deterministic, corruption-free output even when multiple rules
flag overlapping regions.

## Implemented fixes

### R001 — Missing first-line comment

**Diagnostic:** `Missing required first-line Rexx comment`

**Fix:** Inserts `/* The first line of a REXX exec must always be a comment. */`
as the very first line of the file.

```diff
+/* The first line of a REXX exec must always be a comment. */
 SAY 'hello'
```

### R010 — Inconsistent keyword casing

**Diagnostic:** `Inconsistent keyword casing (expected uppercase|lowercase)`

**Fix:** Converts the offending keyword token to the majority casing of the file.

```diff
-say 'hello'
+SAY 'hello'
```

## Fixes that are NOT implemented

| Category | Reason |
|----------|--------|
| Dead code removal | Semantic change |
| Expression rewrites | Semantic change |
| Statement reordering | Semantic change |
| Comment removal | Destructive |
| String content changes | Destructive |

## Exit codes

| Scenario | Exit code |
|----------|-----------|
| No diagnostics, no fixes applied | 0 |
| All diagnostics fixed, none remain | 0 |
| `--fix-only` completed successfully | 0 |
| Diagnostics remain after fixing | 1 |
| Write failure while applying fixes | 2 |
| Bad arguments or I/O error | 2 |
