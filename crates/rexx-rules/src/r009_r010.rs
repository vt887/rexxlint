use rexx_config::default_profile;
use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let profile = default_profile();
    let mut diagnostics = Vec::new();

    for line in &ctx.lines {
        let len = line.raw.chars().count();
        if len > profile.line_length_hard {
            diagnostics.push(Diagnostic {
                code: "R009",
                severity: Severity::Error,
                message: format!(
                    "Line length {} exceeds hard limit {}",
                    len, profile.line_length_hard
                ),
                line: line.line_no,
                column: profile.line_length_hard + 1,
            });
        } else if len > profile.line_length_soft {
            diagnostics.push(Diagnostic {
                code: "R009",
                severity: Severity::Warning,
                message: format!(
                    "Line length {} exceeds soft limit {}",
                    len, profile.line_length_soft
                ),
                line: line.line_no,
                column: profile.line_length_soft + 1,
            });
        }

        if profile.tabs_forbidden {
            for (idx, ch) in line.raw.chars().enumerate() {
                if ch == '\t' {
                    diagnostics.push(Diagnostic {
                        code: "R010",
                        severity: Severity::Error,
                        message: "Tabs are forbidden in mainframe-compatible profile".to_string(),
                        line: line.line_no,
                        column: idx + 1,
                    });
                    break;
                }
            }
        }
    }

    diagnostics
}
