use crate::context::RuleContext;
use rexx_ast::{Program, Statement};
use rexx_diagnostics::Diagnostic;
use rexx_lexer::{Keyword, TokenKind};
use std::collections::HashSet;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut labels = HashSet::new();
    collect_labels(&ctx.program, &mut labels);

    let mut diagnostics = Vec::new();
    check_signals(&ctx.program, &labels, &mut diagnostics);
    diagnostics
}

fn collect_labels(prog: &Program, labels: &mut HashSet<String>) {
    for stmt in &prog.statements {
        collect_labels_stmt(stmt, labels);
    }
}

fn collect_labels_stmt(stmt: &Statement, labels: &mut HashSet<String>) {
    match stmt {
        Statement::Label(l) => {
            labels.insert(l.name.to_ascii_lowercase());
        }
        Statement::DoBlock(b) => {
            for s in &b.body {
                collect_labels_stmt(s, labels);
            }
        }
        Statement::SelectBlock(b) => {
            for case in &b.cases {
                collect_labels_stmt(&case.body, labels);
            }
            if let Some(otherwise) = &b.otherwise {
                for s in &otherwise.body {
                    collect_labels_stmt(s, labels);
                }
            }
        }
        Statement::IfStatement(i) => {
            collect_labels_stmt(&i.then_branch, labels);
            if let Some(else_branch) = &i.else_branch {
                collect_labels_stmt(else_branch, labels);
            }
        }
        _ => {}
    }
}

fn check_signals(prog: &Program, labels: &HashSet<String>, diags: &mut Vec<Diagnostic>) {
    for stmt in &prog.statements {
        check_signals_stmt(stmt, labels, diags);
    }
}

fn check_signals_stmt(stmt: &Statement, labels: &HashSet<String>, diags: &mut Vec<Diagnostic>) {
    match stmt {
        Statement::Command(c) => {
            if let Some(first_tok) = c.tokens.first()
                && let TokenKind::Keyword(Keyword::Signal) = &first_tok.kind
            {
                // Check if it's dynamic or missing target
                if c.tokens.len() > 1 {
                    let second_tok = &c.tokens[1];
                    match &second_tok.kind {
                        TokenKind::Identifier(name) => {
                            if name.eq_ignore_ascii_case("VALUE")
                                || name.eq_ignore_ascii_case("ON")
                                || name.eq_ignore_ascii_case("OFF")
                            {
                                // These are special or dynamic
                                if name.eq_ignore_ascii_case("VALUE") {
                                    diags.push(Diagnostic::warning(
                                        "R008",
                                        "Dynamic SIGNAL target",
                                        c.span,
                                    ));
                                }
                            } else if !labels.contains(&name.to_ascii_lowercase()) {
                                diags.push(Diagnostic::warning(
                                    "R008",
                                    format!("Suspicious SIGNAL target: label '{}' not found", name),
                                    c.span,
                                ));
                            }
                        }
                        TokenKind::LParen => {
                            diags.push(Diagnostic::warning(
                                "R008",
                                "Dynamic SIGNAL target",
                                c.span,
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        Statement::DoBlock(b) => {
            for s in &b.body {
                check_signals_stmt(s, labels, diags);
            }
        }
        Statement::SelectBlock(b) => {
            for case in &b.cases {
                check_signals_stmt(&case.body, labels, diags);
            }
            if let Some(otherwise) = &b.otherwise {
                for s in &otherwise.body {
                    check_signals_stmt(s, labels, diags);
                }
            }
        }
        Statement::IfStatement(i) => {
            check_signals_stmt(&i.then_branch, labels, diags);
            if let Some(else_branch) = &i.else_branch {
                check_signals_stmt(else_branch, labels, diags);
            }
        }
        _ => {}
    }
}
