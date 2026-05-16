use rexx_diagnostics::{Diagnostic, Severity};

use crate::context::RuleContext;

#[derive(Clone, Copy)]
enum Block {
    Do,
    Select,
}

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<(Block, usize)> = Vec::new();

    for line in &ctx.lines {
        let first = line.tokens.first().map(|x| x.lower.as_str());
        match first {
            Some("do") => stack.push((Block::Do, line.line_no)),
            Some("select") => stack.push((Block::Select, line.line_no)),
            Some("end") if stack.pop().is_none() => diagnostics.push(Diagnostic {
                code: "R003",
                severity: Severity::Error,
                message: "Unmatched END".to_string(),
                line: line.line_no,
                column: 1,
            }),
            Some("end") => {}
            _ => {}
        }
    }

    for (block, line_no) in stack {
        let (code, message) = match block {
            Block::Do => ("R003", "Unmatched DO/END"),
            Block::Select => ("R004", "Unmatched SELECT/END"),
        };
        diagnostics.push(Diagnostic {
            code,
            severity: Severity::Error,
            message: message.to_string(),
            line: line_no,
            column: 1,
        });
    }

    diagnostics
}
