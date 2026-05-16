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
    diagnostics.sort_by_key(|d| (d.line, d.column, d.code));
    diagnostics
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::lint;

    fn codes(source: &str) -> Vec<&'static str> {
        lint(source).into_iter().map(|d| d.code).collect()
    }

    #[test]
    fn r001_positive_negative_multiline_and_string_safe() {
        assert!(codes("say 'x'\n").contains(&"R001"));
        assert!(!codes("/* ok */\nsay 'x'\n").contains(&"R001"));
        assert!(!codes("\n\n/* ok */\nsay 'x'\n").contains(&"R001"));
        assert!(!codes("/* comment */\nsay '/* not comment */'\n").contains(&"R001"));
    }

    #[test]
    fn r002_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* broken\nsay 'x'\n").contains(&"R002"));
        assert!(!codes("/* ok */\nsay 'x'\n").contains(&"R002"));
        assert!(!codes("/* a\n b */\nsay 'x'\n").contains(&"R002"));
        assert!(!codes("/* ok */\nsay '/* text */'\n").contains(&"R002"));
    }

    #[test]
    fn r003_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\ndo\nsay 'x'\n").contains(&"R003"));
        assert!(!codes("/* ok */\ndo\nsay 'x'\nend\n").contains(&"R003"));
        assert!(!codes("/* ok */\ndo\ndo\nend\nend\n").contains(&"R003"));
        assert!(!codes("/* ok */\nsay 'do end'\n").contains(&"R003"));
    }

    #[test]
    fn r004_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\nselect\nwhen a=1 then\n say 'x'\n").contains(&"R004"));
        assert!(!codes("/* ok */\nselect\nwhen a=1 then\n say 'x'\nend\n").contains(&"R004"));
        assert!(
            !codes("/* ok */\nselect\nwhen a=1 then\n do\n  say 'x'\n end\nend\n")
                .contains(&"R004")
        );
        assert!(!codes("/* ok */\nsay 'select end'\n").contains(&"R004"));
    }

    #[test]
    fn r005_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\nstart: say 'x'\nSTART: say 'y'\n").contains(&"R005"));
        assert!(!codes("/* ok */\nstart: say 'x'\nnext: say 'y'\n").contains(&"R005"));
        assert!(!codes("/* ok */\nstart:\n say 'x'\nnext:\n say 'y'\n").contains(&"R005"));
        assert!(!codes("/* ok */\nsay 'start:'\n").contains(&"R005"));
    }

    #[test]
    fn r006_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\nexit\nsay 'x'\n").contains(&"R006"));
        assert!(!codes("/* ok */\nreturn\n").contains(&"R006"));
        assert!(!codes("/* ok */\ndo\n return\nend\n").contains(&"R006"));
        assert!(!codes("/* ok */\nsay 'exit return'\n").contains(&"R006"));
    }

    #[test]
    fn r007_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\ninterpret cmd\n").contains(&"R007"));
        assert!(!codes("/* ok */\nsay 'x'\n").contains(&"R007"));
        assert!(!codes("/* ok */\ndo\n interpret cmd\nend\n").is_empty());
        assert!(!codes("/* ok */\nsay 'interpret'\n").contains(&"R007"));
    }

    #[test]
    fn r008_positive_negative_multiline_and_string_safe() {
        assert!(codes("/* ok */\nDO\nend\n").contains(&"R008"));
        assert!(!codes("/* ok */\ndo\nend\n").contains(&"R008"));
        assert!(!codes("/* ok */\nselect\nwhen x = 1 then\n say 'x'\nend\n").contains(&"R008"));
        assert!(!codes("/* ok */\nsay 'DO end'\n").contains(&"R008"));
    }

    #[test]
    fn r009_line_length_soft_and_hard() {
        let soft = format!("/* ok */\n{}\n", "a".repeat(73));
        let hard = format!("/* ok */\n{}\n", "a".repeat(81));
        assert!(codes(&soft).contains(&"R009"));
        assert!(codes(&hard).contains(&"R009"));
    }

    #[test]
    fn r010_tabs_forbidden() {
        assert!(codes("/* ok */\n\tsay 'x'\n").contains(&"R010"));
        assert!(!codes("/* ok */\n    say 'x'\n").contains(&"R010"));
    }

    #[test]
    fn snapshot_rule_mix() {
        let src = "say 'hi'\nDO\nEXIT\nsay 'x'\nEND\nstart: say 1\nstart: say 2\n";
        let joined = lint(src)
            .into_iter()
            .map(|d| format!("{}:{}:{}:{}", d.code, d.line, d.column, d.message))
            .collect::<Vec<_>>()
            .join("\n");
        assert_snapshot!(joined);
    }
}
