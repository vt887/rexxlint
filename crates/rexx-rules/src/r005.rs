use crate::context::RuleContext;
use rexx_ast::{Program, Statement};
use rexx_diagnostics::{Diagnostic, Severity};
use std::collections::HashMap;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut seen = HashMap::new();
    let mut diagnostics = Vec::new();

    traverse_program(&ctx.program, &mut seen, &mut diagnostics);

    diagnostics
}

fn traverse_program(prog: &Program, seen: &mut HashMap<String, u32>, diags: &mut Vec<Diagnostic>) {
    for stmt in &prog.statements {
        traverse_statement(stmt, seen, diags);
    }
}

fn traverse_statement(
    stmt: &Statement,
    seen: &mut HashMap<String, u32>,
    diags: &mut Vec<Diagnostic>,
) {
    match stmt {
        Statement::Label(l) => {
            let key = l.name.to_ascii_lowercase();
            if let Some(first_line) = seen.get(&key) {
                diags.push(Diagnostic {
                    code: "R005".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "Duplicate label '{}' (first seen at line {})",
                        l.name, first_line
                    ),
                    span: l.span,
                    fix: None,
                });
            } else {
                seen.insert(key, l.span.start_line);
            }
        }
        Statement::DoBlock(b) => {
            for s in &b.body {
                traverse_statement(s, seen, diags);
            }
        }
        Statement::SelectBlock(b) => {
            for case in &b.cases {
                traverse_statement(&case.body, seen, diags);
            }
            if let Some(otherwise) = &b.otherwise {
                for s in &otherwise.body {
                    traverse_statement(s, seen, diags);
                }
            }
        }
        Statement::IfStatement(i) => {
            traverse_statement(&i.then_branch, seen, diags);
            if let Some(else_branch) = &i.else_branch {
                traverse_statement(else_branch, seen, diags);
            }
        }
        Statement::Command(_) | Statement::Comment(_) => {}
    }
}
