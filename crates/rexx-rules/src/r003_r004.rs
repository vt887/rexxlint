use crate::context::RuleContext;
use rexx_diagnostics::Diagnostic;

pub fn run(ctx: &RuleContext) -> Vec<Diagnostic> {
    ctx.parser_diagnostics
        .iter()
        .filter(|d| d.code == "R003" || d.code == "R004")
        .cloned()
        .collect()
}
