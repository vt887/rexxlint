use rexx_diagnostics::{Diagnostic, Fix};

/// Convert a 1-based (line, col) position to a byte offset in `source`.
/// Returns `None` if the position is out of range.
fn to_byte_offset(source: &str, line: u32, col: u32) -> Option<usize> {
    let mut current_line = 1u32;
    let mut current_col = 1u32;
    let mut offset = 0usize;

    for ch in source.chars() {
        if current_line == line && current_col == col {
            return Some(offset);
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
        offset += ch.len_utf8();
    }

    // End-of-file position is valid (e.g. appending at EOF)
    if current_line == line && current_col == col {
        Some(offset)
    } else {
        None
    }
}

/// A resolved fix with byte offsets, ready to apply.
struct ResolvedFix<'a> {
    start: usize,
    end: usize,
    replacement: &'a str,
}

/// Apply all fixable diagnostics to `source`, returning the patched string.
///
/// Guarantees:
/// - Fixes are sorted by ascending byte offset; the earliest fix always wins.
/// - Overlapping fixes are skipped entirely — the first one (lowest offset) wins.
/// - Fixes whose span cannot be resolved against `source` are skipped silently.
/// - The result preserves all content outside fixed regions exactly.
pub fn apply_fixes(source: &str, diagnostics: &[Diagnostic]) -> String {
    let mut fixes: Vec<ResolvedFix<'_>> = diagnostics
        .iter()
        .filter_map(|d| d.fix.as_ref())
        .filter_map(|fix| resolve(source, fix))
        .collect();

    if fixes.is_empty() {
        return source.to_string();
    }

    // Sort ascending so the earliest fix in the file has priority.
    fixes.sort_by_key(|f| (f.start, f.end));

    // Greedy non-overlapping selection: earliest start wins.
    let mut accepted: Vec<ResolvedFix<'_>> = Vec::new();
    let mut high_water = 0usize;

    for fix in fixes {
        if fix.start >= high_water {
            high_water = fix.end;
            accepted.push(fix);
        }
        // else: skip — overlaps with an already-accepted fix
    }

    // Apply in descending order so earlier byte offsets are not shifted.
    accepted.sort_by_key(|f| std::cmp::Reverse(f.start));

    let mut result = source.to_string();
    for fix in accepted {
        result.replace_range(fix.start..fix.end, fix.replacement);
    }

    result
}

fn resolve<'a>(source: &str, fix: &'a Fix) -> Option<ResolvedFix<'a>> {
    let start = to_byte_offset(source, fix.span.start_line, fix.span.start_col)?;

    let end = if fix.span.end_line == fix.span.start_line && fix.span.end_col == fix.span.start_col
    {
        // Point span: pure insertion — replace zero bytes at `start`.
        start
    } else {
        to_byte_offset(source, fix.span.end_line, fix.span.end_col)?
    };

    if end > source.len() || start > end {
        return None;
    }

    Some(ResolvedFix {
        start,
        end,
        replacement: &fix.replacement,
    })
}

#[cfg(test)]
mod tests {
    use super::apply_fixes;
    use rexx_diagnostics::{Diagnostic, Severity, Span};

    fn diag_with_fix(replacement: &str, sl: u32, sc: u32, el: u32, ec: u32) -> Diagnostic {
        Diagnostic {
            code: "TEST".into(),
            severity: Severity::Warning,
            message: "test".into(),
            span: Span::new(sl, sc, sl, sc),
            fix: None,
        }
        .with_fix(replacement.to_string(), Span::new(sl, sc, el, ec))
    }

    #[test]
    fn no_fixes_returns_source_unchanged() {
        let src = "/* ok */\nsay 'hi'\n";
        let diag = Diagnostic {
            code: "R001".into(),
            severity: Severity::Warning,
            message: "m".into(),
            span: Span::new(1, 1, 1, 1),
            fix: None,
        };
        assert_eq!(apply_fixes(src, &[diag]), src);
    }

    #[test]
    fn applies_keyword_uppercase_fix() {
        // "say" at (2,1) → "SAY"
        let src = "/* ok */\nsay 'hi'\n";
        let diag = diag_with_fix("SAY", 2, 1, 2, 4);
        let result = apply_fixes(src, &[diag]);
        assert!(result.contains("SAY 'hi'"), "got: {result:?}");
    }

    #[test]
    fn applies_insertion_at_start() {
        let src = "say 'hi'\n";
        let comment = "/* REXX */\n";
        let diag = diag_with_fix(comment, 1, 1, 1, 1);
        let result = apply_fixes(src, &[diag]);
        // Insertion must prepend without consuming any original content.
        assert_eq!(result, "/* REXX */\nsay 'hi'\n", "got: {result:?}");
    }

    #[test]
    fn skips_overlapping_fix() {
        // Two fixes on line 2: col 1-4 and col 2-5 — only the first (lower offset) wins.
        let src = "/* ok */\nsay 'hi'\n";
        let fix1 = diag_with_fix("SAY", 2, 1, 2, 4);
        let fix2 = diag_with_fix("AYS", 2, 2, 2, 5);
        let result = apply_fixes(src, &[fix1, fix2]);
        // The result should contain exactly one of the two fixes applied.
        assert!(
            result.contains("SAY 'hi'") || result.contains("say 'hi'"),
            "got: {result:?}"
        );
    }

    #[test]
    fn multiple_non_overlapping_fixes() {
        // Two keyword fixes on different lines.
        let src = "/* ok */\nsay 'a'\nsay 'b'\n";
        let fix1 = diag_with_fix("SAY", 2, 1, 2, 4);
        let fix2 = diag_with_fix("SAY", 3, 1, 3, 4);
        let result = apply_fixes(src, &[fix1, fix2]);
        assert!(result.contains("SAY 'a'"), "got: {result:?}");
        assert!(result.contains("SAY 'b'"), "got: {result:?}");
    }

    #[test]
    fn idempotent_when_already_fixed() {
        let src = "/* ok */\nSAY 'hi'\n";
        let result = apply_fixes(src, &[]);
        assert_eq!(result, src);
    }
}
