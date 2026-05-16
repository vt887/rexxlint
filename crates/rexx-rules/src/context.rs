use rexx_ast::Program;
use rexx_diagnostics::Diagnostic;
use rexx_lexer::{Lexer, Token};
use rexx_parser::Parser;

#[derive(Debug, Clone)]
pub struct RuleContext {
    pub _source: String,
    pub program: Program,
    pub parser_diagnostics: Vec<Diagnostic>,
    pub tokens: Vec<Token>,
}

pub fn build_context(source: &str) -> RuleContext {
    let lexer = Lexer::new(source);
    let tokens: Vec<Token> = lexer.collect();

    let mut parser = Parser::new(tokens.clone());
    let (program, parser_diagnostics) = parser.parse_program();

    RuleContext {
        _source: source.to_string(),
        program,
        parser_diagnostics,
        tokens,
    }
}
