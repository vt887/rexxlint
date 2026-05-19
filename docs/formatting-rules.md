# Formatting Rules

This document describes every transformation the formatter applies.

## F01 — First-line comment insertion

**Trigger:** File does not start with a block comment (`/* … */`).

**Action:** Inserts a standard header comment as line 1.

**Controlled by:** `insert_first_comment` (default: `true` for mainframe and
standard profiles, `false` for minimal).

**Preserves:** All existing content; the comment is prepended, nothing is removed.

---

## F02 — Keyword casing normalisation

**Trigger:** Any keyword token whose casing does not match the active profile.

**Action:** Rewrites the token text to the target case.

| `keyword_casing` | Example input | Example output |
|------------------|---------------|----------------|
| `upper` | `say 'hi'` | `SAY 'hi'` |
| `lower` | `SAY 'HI'` | `say 'HI'` |
| `preserve` | `Say 'hi'` | `Say 'hi'` (unchanged) |

**Preserves:** String contents, comment text, identifier names.

---

## F03 — Indentation

**Trigger:** Statements inside structured blocks.

**Action:** Indents body statements by `indent_size` spaces × nesting depth.

| Block type | Indented region |
|------------|-----------------|
| `DO … END` | Statements between DO and END |
| `SELECT … END` | WHEN/OTHERWISE bodies |
| `IF … THEN … ELSE` | THEN and ELSE branches |

**Controlled by:** `indent_size` (default: 4). The formatter always uses spaces
for indentation regardless of the `tabs_forbidden` setting; tab enforcement in
existing source lines is not yet implemented.

---

## F04 — Trailing whitespace removal

**Trigger:** Any line that ends with one or more spaces or tabs.

**Action:** Strips trailing whitespace from the line.

**Preserves:** Leading whitespace (indentation), string literals that contain
trailing spaces internally.

---

## F05 — Blank line normalisation

**Trigger:** More than `max_blank_lines` consecutive empty lines.

**Action:** Collapses the run to exactly `max_blank_lines` blank lines.

**Controlled by:** `max_blank_lines` (default: 1 for mainframe and standard, 2 for
minimal).

---

## F06 — Trailing newline

**Trigger:** File does not end with exactly one newline.

**Action:** Ensures the file ends with `\n`. Trailing blank lines beyond the last
code line are stripped.

---

## Unsupported transformations

The following are explicitly **not** performed:

| Transformation | Reason |
|----------------|--------|
| Line splitting for long lines | Continuation semantics are complex; risk of semantic change |
| Comment reformatting | Comments are user-authored prose |
| String content changes | Destructive |
| Statement reordering | Semantic change |
| Dead code removal | Semantic change |
| Expression simplification | Semantic change |
| Tab expansion inside strings | Destructive |

## Configuration reference

```toml
[formatting]
indent_size = 4              # spaces per indent level
keyword_casing = "upper"    # "upper" | "lower" | "preserve"
max_blank_lines = 1          # max consecutive blank lines
insert_first_comment = true  # prepend header comment if missing
tabs_forbidden = true        # treat tabs in indentation as an error
line_length_soft = 72        # advisory line length (stored in profile; not yet enforced)
line_length_hard = 80        # hard line length (stored in profile; not yet enforced)
```

> **Deprecated field:** `uppercase_keywords = true/false` was the original boolean
> equivalent of `keyword_casing`. It is still accepted for backwards compatibility
> and maps `true` → `"upper"`, `false` → `"lower"`, but a deprecation warning is
> printed. Replace it with `keyword_casing` and remove the old field.
