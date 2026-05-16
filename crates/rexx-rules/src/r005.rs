use std::collections::BTreeMap;

use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for line in &ctx.lines {
        let trimmed = line.text.trim_start();
        let mut chars = trimmed.chars().peekable();
        let mut label = String::new();
        while let Some(ch) = chars.peek().copied() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                label.push(ch);
                let _ = chars.next();
            } else {
                break;
            }
        }
        if !label.is_empty() && chars.peek() == Some(&':') {
            let key = label.to_ascii_lowercase();
            if let Some(first_line) = seen.get(&key) {
                diagnostics.push(Diagnostic {
                    code: "R005",
                    severity: Severity::Warning,
                    message: format!("Duplicate label '{label}' (first seen at line {first_line})"),
                    line: line.line_no,
                    column: 1,
                });
            } else {
                seen.insert(key, line.line_no);
            }
        }
    }

    diagnostics
}
