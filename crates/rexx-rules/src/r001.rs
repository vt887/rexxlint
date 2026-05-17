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

    let comment = "/* The first line of a REXX exec must always be a comment. */\n";
    vec![
        Diagnostic {
            code: "R001".to_string(),
            severity: Severity::Error,
            message: "Missing required first-line Rexx comment".to_string(),
            span: Span::new(1, 1, 1, 1),
            fix: None,
        }
        .with_fix(comment.to_string(), Span::new(1, 1, 1, 1)),
    ]
}
