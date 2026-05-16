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
    check_statements(&prog.statements, diags);
    for stmt in &prog.statements {
        traverse_statement(stmt, diags);
    }
}

fn check_statements(stmts: &[Statement], diags: &mut Vec<Diagnostic>) {
    let mut exit_found = false;
    for stmt in stmts {
        if exit_found && !matches!(stmt, Statement::Label(_) | Statement::Comment(_)) {
            diags.push(Diagnostic::warning(
                "R006",
                "Unreachable code after EXIT/RETURN",
                stmt.span(),
            ));
            // Only flag the first unreachable statement to avoid noise
            exit_found = false;
        }

        if let Statement::Command(c) = stmt
            && let Some(first_tok) = c.tokens.first()
            && let TokenKind::Keyword(k) = &first_tok.kind
            && matches!(k, Keyword::Exit | Keyword::Return)
        {
            exit_found = true;
        }
    }
}
fn traverse_statement(stmt: &Statement, diags: &mut Vec<Diagnostic>) {
    match stmt {
        Statement::DoBlock(b) => {
            check_statements(&b.body, diags);
            for s in &b.body {
                traverse_statement(s, diags);
            }
        }
        Statement::SelectBlock(b) => {
            for case in &b.cases {
                if let Statement::Command(_)
                | Statement::DoBlock(_)
                | Statement::SelectBlock(_)
                | Statement::IfStatement(_) = &*case.body
                {
                    check_statements(std::slice::from_ref(&case.body), diags);
                }
                traverse_statement(&case.body, diags);
            }
            if let Some(otherwise) = &b.otherwise {
                check_statements(&otherwise.body, diags);
                for s in &otherwise.body {
                    traverse_statement(s, diags);
                }
            }
        }
        Statement::IfStatement(i) => {
            check_statements(std::slice::from_ref(&i.then_branch), diags);
            traverse_statement(&i.then_branch, diags);
            if let Some(else_branch) = &i.else_branch {
                check_statements(std::slice::from_ref(else_branch), diags);
                traverse_statement(else_branch, diags);
            }
        }
        _ => {}
    }
}
