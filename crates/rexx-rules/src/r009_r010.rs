use crate::context::RuleContext;
use rexx_diagnostics::Diagnostic;
use rexx_lexer::TokenKind;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    check_r009(ctx, &mut diagnostics);
    check_r010(ctx, &mut diagnostics);
    diagnostics
}

fn check_r009(ctx: &RuleContext, diags: &mut Vec<Diagnostic>) {
    for token in &ctx.tokens {
        if matches!(token.kind, TokenKind::Continuation) {
            let mut substantive_after = false;
            let mut found_current = false;
            for t in &ctx.tokens {
                if !found_current {
                    if std::ptr::eq(t, token) {
                        found_current = true;
                    }
                    continue;
                }
                match &t.kind {
                    TokenKind::Newline
                    | TokenKind::Semicolon
                    | TokenKind::LineComment(_)
                    | TokenKind::BlockComment(_) => {}
                    _ => {
                        substantive_after = true;
                        break;
                    }
                }
            }
            if !substantive_after {
                diags.push(Diagnostic::warning(
                    "R009",
                    "Trailing continuation comma with no following content",
                    rexx_diagnostics::Span::new(token.line, token.col, token.line, token.col),
                ));
            }
        }
    }
}

fn check_r010(ctx: &RuleContext, diags: &mut Vec<Diagnostic>) {
    let mut upper_count = 0;
    let mut lower_count = 0;
    let mut keywords = Vec::new();

    for token in &ctx.tokens {
        if let TokenKind::Keyword(_) = &token.kind {
            let is_upper = token
                .text
                .chars()
                .all(|c| !c.is_alphabetic() || c.is_uppercase());
            let is_lower = token
                .text
                .chars()
                .all(|c| !c.is_alphabetic() || c.is_lowercase());

            if is_upper {
                upper_count += 1;
            } else if is_lower {
                lower_count += 1;
            }
            keywords.push((token, is_upper, is_lower));
        }
    }

    if upper_count > 0 && lower_count > 0 {
        // Preference is for majority or uppercase
        let prefer_upper = upper_count >= lower_count;
        for (token, is_upper, is_lower) in keywords {
            if prefer_upper && !is_upper {
                diags.push(
                    Diagnostic::warning(
                        "R010",
                        "Inconsistent keyword casing (expected uppercase)",
                        rexx_diagnostics::Span::new(token.line, token.col, token.line, token.col),
                    )
                    .with_fix(
                        token.text.to_ascii_uppercase(),
                        rexx_diagnostics::Span::new(token.line, token.col, token.line, token.col),
                    ),
                );
            } else if !prefer_upper && !is_lower {
                diags.push(
                    Diagnostic::warning(
                        "R010",
                        "Inconsistent keyword casing (expected lowercase)",
                        rexx_diagnostics::Span::new(token.line, token.col, token.line, token.col),
                    )
                    .with_fix(
                        token.text.to_ascii_lowercase(),
                        rexx_diagnostics::Span::new(token.line, token.col, token.line, token.col),
                    ),
                );
            }
        }
    }
}
