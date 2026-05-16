use crate::context::RuleContext;
use rexx_diagnostics::{Diagnostic, Severity, Span};
use rexx_lexer::TokenKind;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    ctx.tokens
        .iter()
        .filter_map(|t| {
            if matches!(t.kind, TokenKind::UnterminatedBlockComment(_)) {
                Some(Diagnostic {
                    code: "R002".to_string(),
                    severity: Severity::Error,
                    message: "Unclosed block comment".to_string(),
                    span: Span::new(t.line, t.col, t.line, t.col),
                    fix: None,
                })
            } else {
                None
            }
        })
        .collect()
}
