use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut depth = 0usize;
    let mut terminated_at_depth: Option<usize> = None;

    for line in &ctx.lines {
        let first = line.tokens.first().map(|x| x.lower.as_str());

        if matches!(first, Some("end")) {
            depth = depth.saturating_sub(1);
            if terminated_at_depth.is_some_and(|d| depth < d) {
                terminated_at_depth = None;
            }
            continue;
        }

        if terminated_at_depth.is_some()
            && !line.tokens.is_empty()
            && !matches!(first, Some("when") | Some("otherwise"))
        {
            diagnostics.push(Diagnostic {
                code: "R006",
                severity: Severity::Warning,
                message: "Unreachable code after EXIT/RETURN".to_string(),
                line: line.line_no,
                column: 1,
            });
            terminated_at_depth = None;
        }

        if matches!(first, Some("do") | Some("select")) {
            depth += 1;
        }

        if matches!(first, Some("exit") | Some("return")) {
            terminated_at_depth = Some(depth);
        }
    }

    diagnostics
}
