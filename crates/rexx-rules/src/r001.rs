use crate::context::RuleContext;
use rexx_diagnostics::{Diagnostic, Severity, Span};
use rexx_lexer::TokenKind;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    // Find first non-newline token
    for token in &ctx.tokens {
        if matches!(token.kind, TokenKind::Newline) {
            continue;
        }
        if matches!(token.kind, TokenKind::BlockComment(_)) {
            return Vec::new();
        } else {
            break;
        }
    }

    vec![Diagnostic {
        code: "R001".to_string(),
        severity: Severity::Error,
        message: "Missing required first-line Rexx comment".to_string(),
        span: Span::new(1, 1, 1, 1),
        fix: None,
    }]
}
