use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    if let Some(line) = ctx.source.lines().find(|line| !line.trim().is_empty()) {
        let t = line.trim();
        if !(t.starts_with("/*") && t.ends_with("*/")) {
            return vec![Diagnostic {
                code: "R001",
                severity: Severity::Error,
                message: "Missing required first-line Rexx comment".to_string(),
                line: 1,
                column: 1,
            }];
        }
    }
    Vec::new()
}
