use crate::context::RuleContext;
use rexx_ast::{Program, Statement};
use rexx_diagnostics::Diagnostic;
use rexx_lexer::{Keyword, TokenKind};

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    traverse_program(&ctx.program, &mut diagnostics);
    diagnostics
}

fn traverse_program(prog: &Program, diags: &mut Vec<Diagnostic>) {
    for stmt in &prog.statements {
        traverse_statement(stmt, diags);
    }
}

fn traverse_statement(stmt: &Statement, diags: &mut Vec<Diagnostic>) {
    match stmt {
        Statement::Command(c) => {
            if let Some(first_tok) = c.tokens.first()
                && let TokenKind::Keyword(Keyword::Interpret) = &first_tok.kind
            {
                diags.push(Diagnostic::warning(
                    "R007",
                    "Unsafe INTERPRET usage",
                    c.span,
                ));
            }
        }
        Statement::DoBlock(b) => {
            for s in &b.body {
                traverse_statement(s, diags);
            }
        }
        Statement::SelectBlock(b) => {
            for case in &b.cases {
                traverse_statement(&case.body, diags);
            }
            if let Some(otherwise) = &b.otherwise {
                for s in &otherwise.body {
                    traverse_statement(s, diags);
                }
            }
        }
        Statement::IfStatement(i) => {
            traverse_statement(&i.then_branch, diags);
            if let Some(else_branch) = &i.else_branch {
                traverse_statement(else_branch, diags);
            }
        }
        _ => {}
    }
}
