use insta::assert_snapshot;
use rexx_analyzer::lint;
use rexx_cli::{render_json, render_sarif, render_text};
use rexx_formatter::format_rexx;

#[test]
fn snapshot_text_output() {
    let src = "say 'hi'\nDO\nEXIT\nsay 'x'\nEND\n";
    let diagnostics = lint(src);
    let out = render_text("sample.rexx", &diagnostics);
    assert_snapshot!(out);
}

#[test]
fn snapshot_json_output() {
    let src = "say 'hi'\nDO\nEXIT\nsay 'x'\nEND\n";
    let diagnostics = lint(src);
    let out = render_json(&diagnostics).expect("json render");
    assert_snapshot!(out);
}

#[test]
fn snapshot_sarif_output() {
    let src = "say 'hi'\nDO\nEXIT\nsay 'x'\nEND\n";
    let diagnostics = lint(src);
    let out = render_sarif("sample.rexx", &diagnostics).expect("sarif render");
    assert_snapshot!(out);
}

#[test]
fn snapshot_formatter_before_after() {
    let before = "say 'Hello'\n\tDO\n\t\tSay 'World'\n\tEND\n";
    let after = format_rexx(before);
    assert_snapshot!(format!("--- BEFORE ---\n{before}\n--- AFTER ---\n{after}"));
}
