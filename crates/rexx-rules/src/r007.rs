use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for line in &ctx.lines {
        if let Some(token) = line.tokens.iter().find(|t| t.lower == "interpret") {
            diagnostics.push(Diagnostic {
                code: "R007",
                severity: Severity::Warning,
                message: "Unsafe INTERPRET usage".to_string(),
                line: line.line_no,
                column: token.column,
            });
        }
    }
    diagnostics
}
