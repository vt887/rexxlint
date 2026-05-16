mod context;
mod r001;
mod r002;
mod r003_r004;
mod r005;
mod r006;
mod r007;
mod r008;
mod r009_r010;

use context::build_context;
use rexx_diagnostics::Diagnostic;

pub fn lint(source: &str) -> Vec<Diagnostic> {
    let ctx = build_context(source);
    let mut diagnostics = Vec::new();
    diagnostics.extend(r001::run(&ctx));
    diagnostics.extend(r002::run(&ctx));
    diagnostics.extend(r003_r004::run(&ctx));
    diagnostics.extend(r005::run(&ctx));
    diagnostics.extend(r006::run(&ctx));
    diagnostics.extend(r007::run(&ctx));
    diagnostics.extend(r008::run(&ctx));
    diagnostics.extend(r009_r010::run(&ctx));
    diagnostics.sort_by_key(|d| (d.span.start_line, d.span.start_col, d.code.clone()));
    diagnostics
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::lint;

    fn has(source: &str, code: &str) -> bool {
        lint(source).into_iter().any(|d| d.code == code)
    }

    #[test]
    fn r001_positive_negative_multiline_and_string_safe() {
        assert!(has("say 'x'\n", "R001"));
        assert!(!has("/* ok */\nsay 'x'\n", "R001"));
        assert!(!has("\n\n/* ok */\nsay 'x'\n", "R001"));
        assert!(!has("/* comment */\nsay '/* not comment */'\n", "R001"));
    }

    #[test]
    fn r002_positive_negative_multiline_and_string_safe() {
        assert!(has("/* broken\nsay 'x'\n", "R002"));
        assert!(!has("/* ok */\nsay 'x'\n", "R002"));
        assert!(!has("/* a\n b */\nsay 'x'\n", "R002"));
        assert!(!has("/* ok */\nsay '/* text */'\n", "R002"));
    }

    #[test]
    fn r003_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */\ndo\nsay 'x'\n", "R003"));
        assert!(!has("/* ok */\ndo\nsay 'x'\nend\n", "R003"));
        assert!(!has("/* ok */\ndo\ndo\nend\nend\n", "R003"));
        assert!(!has("/* ok */\nsay 'do end'\n", "R003"));
    }

    #[test]
    fn r004_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */\nselect\nwhen a=1 then\n say 'x'\n", "R004"));
        assert!(!has(
            "/* ok */\nselect\nwhen a=1 then\n say 'x'\nend\n",
            "R004"
        ));
        assert!(!has(
            "/* ok */\nselect\nwhen a=1 then\n do\n  say 'x'\n end\nend\n",
            "R004"
        ));
        assert!(!has("/* ok */\nsay 'select end'\n", "R004"));
    }

    #[test]
    fn r005_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */\nstart: say 'x'\nSTART: say 'y'\n", "R005"));
        assert!(!has("/* ok */\nstart: say 'x'\nnext: say 'y'\n", "R005"));
        assert!(!has(
            "/* ok */\nstart:\n say 'x'\nnext:\n say 'y'\n",
            "R005"
        ));
        assert!(!has("/* ok */\nsay 'start:'\n", "R005"));
    }

    /*
    #[test]
    fn r006_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */
\nexit\nsay 'x'\n", "R006"));
    assert!(!has(" /* ok */
\nreturn\n", "R006"));
    assert!(!has(" /* ok */
\ndo\n return\nend\n", "R006"));
    assert!(!has(" /* ok */
\nsay 'exit return'\n", "R006"));
    }

    #[test]
    fn r007_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */\ninterpret cmd\n", "R007"));
        assert!(!has("/* ok */\nsay 'x'\n", "R007"));
        assert!(!lint("/* ok */\ndo\n interpret cmd\nend\n").is_empty());
        assert!(!has("/* ok */\nsay 'interpret'\n", "R007"));
    }

    #[test]
    fn r008_positive_negative_multiline_and_string_safe() {
        assert!(has("/* ok */\nDO\nend\n", "R008"));
        assert!(!has("/* ok */\ndo\nend\n", "R008"));
        assert!(!has("/* ok */\nselect\nwhen x = 1 then\n say 'x'\nend\n", "R008"));
        assert!(!has("/* ok */\nsay 'DO end'\n", "R008"));
    }

    #[test]
    fn r009_line_length_soft_and_hard() {
        let soft = format!("/* ok */\n{}\n", "a".repeat(73));
        let hard = format!("/* ok */\n{}\n", "a".repeat(81));
        assert!(has(&soft, "R009"));
        assert!(has(&hard, "R009"));
    }

    #[test]
    fn r010_tabs_forbidden() {
        assert!(has("/* ok */\n\tsay 'x'\n", "R010"));
        assert!(!has("/* ok */\n    say 'x'\n", "R010"));
    }
    */

    #[test]
    fn snapshot_rule_mix() {
        let src = "say 'hi'\nDO\nEXIT\nsay 'x'\nEND\nstart: say 1\nstart: say 2\n";
        let joined = lint(src)
            .into_iter()
            .map(|d| {
                format!(
                    "{}:{}:{}:{}",
                    d.code, d.span.start_line, d.span.start_col, d.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_snapshot!(joined);
    }
}
