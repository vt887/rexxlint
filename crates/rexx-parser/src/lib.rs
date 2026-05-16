use rexx_ast::*;
use rexx_diagnostics::{Diagnostic, Span};
use rexx_lexer::{Keyword, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(tok)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn span_from(&self, start_token: &Token) -> Span {
        let last_token = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .unwrap_or(start_token);
        Span::new(
            start_token.line,
            start_token.col,
            last_token.line,
            last_token.col, // This is technically start of last token, should probably be end
        )
    }

    fn token_span(&self, token: &Token) -> Span {
        // Simplified span: start and end are same token pos for now
        Span::new(token.line, token.col, token.line, token.col)
    }

    pub fn parse_program(&mut self) -> (Program, Vec<Diagnostic>) {
        let mut statements = Vec::new();
        let start_line = self.peek().map(|t| t.line).unwrap_or(1);
        let start_col = self.peek().map(|t| t.col).unwrap_or(1);

        while !self.is_at_end() {
            self.consume_newlines_and_semicolons();
            if self.is_at_end() {
                break;
            }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.advance();
            }
        }

        let end_line = self.tokens.last().map(|t| t.line).unwrap_or(start_line);
        let end_col = self.tokens.last().map(|t| t.col).unwrap_or(start_col);

        (
            Program {
                statements,
                span: Span::new(start_line, start_col, end_line, end_col),
            },
            self.diagnostics.clone(),
        )
    }

    fn consume_newlines_and_semicolons(&mut self) {
        while let Some(kind) = self.peek_kind() {
            if matches!(kind, TokenKind::Newline | TokenKind::Semicolon) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let tok = self.peek()?;
        match &tok.kind {
            TokenKind::LineComment(_)
            | TokenKind::BlockComment(_)
            | TokenKind::UnterminatedBlockComment(_) => {
                let tok = self.advance()?.clone();
                Some(Statement::Comment(tok))
            }
            TokenKind::Label(name) => {
                let name = name.clone();
                let span = self.token_span(tok);
                self.advance();
                Some(Statement::Label(Label { name, span }))
            }
            TokenKind::Keyword(Keyword::Do) => self.parse_do_block().map(Statement::DoBlock),
            TokenKind::Keyword(Keyword::Select) => {
                self.parse_select_block().map(Statement::SelectBlock)
            }
            TokenKind::Keyword(Keyword::If) => {
                self.parse_if_statement().map(Statement::IfStatement)
            }
            _ => self.parse_command().map(Statement::Command),
        }
    }

    fn consume_delimiters(&mut self) {
        self.consume_newlines_and_semicolons();
    }

    fn parse_do_block(&mut self) -> Option<DoBlock> {
        let start_token = self.advance()?.clone();
        let mut header_tokens = vec![start_token.clone()];

        // Consume header until delimiter
        while let Some(tok) = self.peek() {
            if matches!(tok.kind, TokenKind::Newline | TokenKind::Semicolon) {
                break;
            }
            header_tokens.push(self.advance()?.clone());
        }

        let header_span = self.span_from(&start_token);
        let header = Command {
            tokens: header_tokens,
            span: header_span,
        };

        let mut body = Vec::new();
        let mut footer_tokens = Vec::new();

        while !self.is_at_end() {
            self.consume_delimiters();
            if let Some(TokenKind::Keyword(Keyword::End)) = self.peek_kind() {
                let end_tok = self.advance()?.clone();
                footer_tokens.push(end_tok);
                // Optionally consume what's after END (e.g., END i)
                while let Some(tok) = self.peek() {
                    if matches!(tok.kind, TokenKind::Newline | TokenKind::Semicolon) {
                        break;
                    }
                    footer_tokens.push(self.advance()?.clone());
                }
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                self.advance();
            }
        }

        if footer_tokens.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "R003",
                "Unmatched DO: missing END",
                header_span,
            ));
        }

        let full_span = self.span_from(&start_token);
        Some(DoBlock {
            header,
            body,
            footer_tokens,
            span: full_span,
        })
    }

    fn parse_select_block(&mut self) -> Option<SelectBlock> {
        let start_token = self.advance()?.clone();
        let header_span = self.token_span(&start_token);

        let mut cases = Vec::new();
        let mut otherwise = None;
        let mut footer_tokens = Vec::new();

        while !self.is_at_end() {
            self.consume_delimiters();
            match self.peek_kind() {
                Some(TokenKind::Keyword(Keyword::When)) => {
                    if let Some(case) = self.parse_when_case() {
                        cases.push(case);
                    }
                }
                Some(TokenKind::Keyword(Keyword::Otherwise)) => {
                    otherwise = self.parse_otherwise_case();
                }
                Some(TokenKind::Keyword(Keyword::End)) => {
                    let end_tok = self.advance()?.clone();
                    footer_tokens.push(end_tok);
                    while let Some(tok) = self.peek() {
                        if matches!(tok.kind, TokenKind::Newline | TokenKind::Semicolon) {
                            break;
                        }
                        footer_tokens.push(self.advance()?.clone());
                    }
                    break;
                }
                _ => {
                    // unexpected inside select, but we can try to parse it as statement or skip
                    if let Some(stmt) = self.parse_statement() {
                        // This shouldn't really happen in valid REXX outside WHEN/OTHERWISE
                        // but let's keep it for now.
                        self.diagnostics.push(Diagnostic::warning(
                            "W001",
                            "Statement outside WHEN/OTHERWISE in SELECT",
                            stmt.span(),
                        ));
                    } else {
                        self.advance();
                    }
                }
            }
        }

        if footer_tokens.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "R004",
                "Unmatched SELECT: missing END",
                header_span,
            ));
        }

        let full_span = self.span_from(&start_token);
        Some(SelectBlock {
            header_span,
            cases,
            otherwise,
            footer_tokens,
            span: full_span,
        })
    }

    fn parse_when_case(&mut self) -> Option<WhenCase> {
        let start_token = self.advance()?.clone();
        let mut cond_tokens = Vec::new();

        while let Some(tok) = self.peek() {
            if matches!(tok.kind, TokenKind::Keyword(Keyword::Then)) {
                break;
            }
            cond_tokens.push(self.advance()?.clone());
        }

        if let Some(TokenKind::Keyword(Keyword::Then)) = self.peek_kind() {
            self.advance();
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E003",
                "WHEN missing THEN",
                self.token_span(&start_token),
            ));
        }

        let condition_span = self.span_from(&start_token);
        let condition = Command {
            tokens: cond_tokens,
            span: condition_span,
        };

        self.consume_delimiters();
        let body = Box::new(
            self.parse_statement()
                .unwrap_or(Statement::Command(Command {
                    tokens: Vec::new(),
                    span: self.span_from(&start_token),
                })),
        );

        Some(WhenCase {
            condition,
            body,
            span: self.span_from(&start_token),
        })
    }

    fn parse_otherwise_case(&mut self) -> Option<OtherwiseCase> {
        let start_token = self.advance()?.clone();
        let mut body = Vec::new();

        while !self.is_at_end() {
            self.consume_delimiters();
            if matches!(self.peek_kind(), Some(TokenKind::Keyword(Keyword::End))) {
                break;
            }
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                break;
            }
        }

        Some(OtherwiseCase {
            body,
            span: self.span_from(&start_token),
        })
    }

    fn parse_if_statement(&mut self) -> Option<IfStatement> {
        let start_token = self.advance()?.clone();
        let mut cond_tokens = Vec::new();

        while let Some(tok) = self.peek() {
            if matches!(tok.kind, TokenKind::Keyword(Keyword::Then)) {
                break;
            }
            cond_tokens.push(self.advance()?.clone());
        }

        if let Some(TokenKind::Keyword(Keyword::Then)) = self.peek_kind() {
            self.advance();
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E004",
                "IF missing THEN",
                self.token_span(&start_token),
            ));
        }

        let condition_span = self.span_from(&start_token);
        let condition = Command {
            tokens: cond_tokens,
            span: condition_span,
        };

        self.consume_delimiters();
        let then_branch = Box::new(self.parse_statement()?);

        self.consume_delimiters();
        let mut else_branch = None;
        if let Some(TokenKind::Keyword(Keyword::Else)) = self.peek_kind() {
            self.advance();
            self.consume_delimiters();
            else_branch = self.parse_statement().map(Box::new);
        }

        Some(IfStatement {
            condition,
            then_branch,
            else_branch,
            span: self.span_from(&start_token),
        })
    }

    fn parse_command(&mut self) -> Option<Command> {
        let start_token = self.peek()?.clone();
        let mut tokens = Vec::new();

        while let Some(tok) = self.peek() {
            if matches!(tok.kind, TokenKind::Newline | TokenKind::Semicolon) {
                break;
            }
            tokens.push(self.advance()?.clone());
        }

        let span = self.span_from(&start_token);
        Some(Command { tokens, span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use rexx_lexer::Lexer;

    fn parse(src: &str) -> (Program, Vec<Diagnostic>) {
        let tokens: Vec<Token> = Lexer::new(src).collect();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    #[test]
    fn test_parse_simple_command() {
        let (prog, diags) = parse("say 'hello'");
        assert!(diags.is_empty());
        assert_eq!(prog.statements.len(), 1);
        if let Statement::Command(cmd) = &prog.statements[0] {
            assert_eq!(cmd.tokens.len(), 2);
        } else {
            panic!("Expected Command");
        }
    }

    #[test]
    fn test_parse_do_end() {
        let (prog, diags) = parse("do\n  say 'hi'\nend");
        assert!(diags.is_empty());
        assert_eq!(prog.statements.len(), 1);
        assert_debug_snapshot!(prog);
    }

    #[test]
    fn test_parse_select() {
        let (prog, diags) = parse("select\n when a=1 then say 1\n otherwise\n  say 0\nend");
        assert!(diags.is_empty());
        assert_debug_snapshot!(prog);
    }

    #[test]
    fn test_parse_if_then_else() {
        let (prog, diags) = parse("if a then say 1; else say 2");
        assert!(diags.is_empty());
        assert_debug_snapshot!(prog);
    }

    #[test]
    fn test_unmatched_do() {
        let (_, diags) = parse("do\n say 'unmatched'");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "R003");
    }

    #[test]
    fn test_label_and_command() {
        let (prog, diags) = parse("START:\n say 'started'");
        assert!(diags.is_empty());
        assert_eq!(prog.statements.len(), 2);
        assert_debug_snapshot!(prog);
    }
}
