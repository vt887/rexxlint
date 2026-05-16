use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

const KEYWORDS: &[&str] = &[
    "do",
    "end",
    "select",
    "when",
    "then",
    "otherwise",
    "if",
    "else",
    "return",
    "exit",
    "interpret",
];

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut baseline_upper: Option<bool> = None;

    for line in &ctx.lines {
        for token in &line.tokens {
            if !KEYWORDS.contains(&token.lower.as_str()) {
                continue;
            }
            let is_upper = token
                .text
                .chars()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_uppercase());
            let is_lower = token
                .text
                .chars()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase());
            if !is_upper && !is_lower {
                diagnostics.push(Diagnostic {
                    code: "R008",
                    severity: Severity::Warning,
                    message: "Inconsistent keyword casing".to_string(),
                    line: line.line_no,
                    column: token.column,
                });
                continue;
            }
            if let Some(base) = baseline_upper {
                if base != is_upper {
                    diagnostics.push(Diagnostic {
                        code: "R008",
                        severity: Severity::Warning,
                        message: "Inconsistent keyword casing".to_string(),
                        line: line.line_no,
                        column: token.column,
                    });
                }
            } else {
                baseline_upper = Some(is_upper);
            }
        }
    }

    diagnostics
}
