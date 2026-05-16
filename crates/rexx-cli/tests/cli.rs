use std::fs;

#[test]
fn smoke_lint_json() {
    let tmp = std::env::temp_dir().join("rexxlint-smoke.rexx");
    fs::write(&tmp, "say 'hi'\n").expect("write temp file");
    let source = fs::read_to_string(&tmp).expect("read temp file");
    let diagnostics = rexx_analyzer::lint(&source);
    assert!(diagnostics.iter().any(|d| d.code == "R001"));
}
