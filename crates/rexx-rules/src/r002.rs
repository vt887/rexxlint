use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    if ctx.has_unclosed_comment {
        vec![Diagnostic {
            code: "R002",
            severity: Severity::Error,
            message: "Unclosed block comment".to_string(),
            line: 1,
            column: 1,
        }]
    } else {
        Vec::new()
    }
}
