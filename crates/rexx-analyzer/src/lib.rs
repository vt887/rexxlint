use rexx_diagnostics::Diagnostic;

pub fn lint(source: &str) -> Vec<Diagnostic> {
    rexx_rules::lint(source)
}
