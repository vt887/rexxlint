pub mod fix_applicator;
pub use fix_applicator::apply_fixes;

use rexx_ast::*;
use rexx_config::{ConfigError, FormattingProfile, KeywordCasing, default_profile, load_profile};
use rexx_lexer::{Lexer, Token, TokenKind};
use rexx_parser::Parser;

pub fn format_rexx(input: &str) -> String {
    format_rexx_with_profile(input, default_profile())
}

pub fn format_rexx_with_profile_name(
    input: &str,
    profile_name: &str,
) -> Result<String, ConfigError> {
    let profile = load_profile(profile_name)?;
    Ok(format_rexx_with_profile(input, profile))
}

pub fn format_rexx_with_profile(input: &str, profile: FormattingProfile) -> String {
    let tokens: Vec<Token> = Lexer::new(input).collect();
    let mut parser = Parser::new(tokens);
    let (prog, _diags) = parser.parse_program();
    let mut formatter = Formatter::new(profile);
    let raw = formatter.format(&prog);
    normalize_output(&raw, formatter.profile.max_blank_lines)
}

struct Formatter {
    profile: FormattingProfile,
    indent_level: usize,
    output: Vec<String>,
}

impl Formatter {
    fn new(profile: FormattingProfile) -> Self {
        Self {
            profile,
            indent_level: 0,
            output: Vec::new(),
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.profile.indent_size)
    }

    fn kw(&self, word: &str) -> String {
        match self.profile.keyword_casing {
            KeywordCasing::Upper => word.to_ascii_uppercase(),
            KeywordCasing::Lower => word.to_ascii_lowercase(),
            KeywordCasing::Preserve => word.to_string(),
        }
    }

    fn format(&mut self, prog: &Program) -> String {
        if prog.statements.is_empty() {
            if self.profile.insert_first_comment {
                return "/* The first line of a REXX exec must always be a comment. */\n"
                    .to_string();
            }
            return String::new();
        }

        let has_first_comment = matches!(prog.statements.first(), Some(Statement::Comment(_)));

        if !has_first_comment && self.profile.insert_first_comment {
            self.output
                .push("/* The first line of a REXX exec must always be a comment. */".to_string());
        }

        for stmt in &prog.statements {
            self.format_statement(stmt);
        }

        self.output.join("\n") + "\n"
    }

    fn format_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Comment(tok) => {
                self.output.push(format!("{}{}", self.indent(), tok.text));
            }
            Statement::Label(l) => {
                self.output.push(format!("{}:", l.name));
            }
            Statement::Command(cmd) => {
                let text = self.format_command(cmd);
                for line in text.lines() {
                    self.output.push(format!("{}{}", self.indent(), line));
                }
            }
            Statement::DoBlock(b) => {
                let header = self.format_command(&b.header);
                self.output.push(format!("{}{}", self.indent(), header));
                self.indent_level += 1;
                for s in &b.body {
                    self.format_statement(s);
                }
                self.indent_level -= 1;
                let footer = self.format_tokens(&b.footer_tokens);
                self.output.push(format!("{}{}", self.indent(), footer));
            }
            Statement::SelectBlock(b) => {
                let select_kw = self.kw("select");
                self.output.push(format!("{}{}", self.indent(), select_kw));
                self.indent_level += 1;
                for when in &b.cases {
                    let wh = self.kw("when");
                    let cond = self.format_command(&when.condition);
                    let th = self.kw("then");
                    self.output
                        .push(format!("{}{} {} {}", self.indent(), wh, cond, th));
                    self.indent_level += 1;
                    self.format_statement(&when.body);
                    self.indent_level -= 1;
                }
                if let Some(otherwise) = &b.otherwise {
                    let ow = self.kw("otherwise");
                    self.output.push(format!("{}{}", self.indent(), ow));
                    self.indent_level += 1;
                    for s in &otherwise.body {
                        self.format_statement(s);
                    }
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
                let footer = self.format_tokens(&b.footer_tokens);
                self.output.push(format!("{}{}", self.indent(), footer));
            }
            Statement::IfStatement(i) => {
                let cond = self.format_command(&i.condition);
                let if_kw = self.kw("if");
                let then_kw = self.kw("then");
                self.output
                    .push(format!("{}{} {} {}", self.indent(), if_kw, cond, then_kw));
                self.indent_level += 1;
                self.format_statement(&i.then_branch);
                self.indent_level -= 1;
                if let Some(else_branch) = &i.else_branch {
                    let else_kw = self.kw("else");
                    self.output.push(format!("{}{}", self.indent(), else_kw));
                    self.indent_level += 1;
                    self.format_statement(else_branch);
                    self.indent_level -= 1;
                }
            }
        }
    }

    fn format_command(&self, cmd: &Command) -> String {
        self.format_tokens(&cmd.tokens)
    }

    fn format_tokens(&self, tokens: &[Token]) -> String {
        let mut parts = Vec::new();
        for t in tokens {
            let part = match &t.kind {
                TokenKind::Keyword(k) => match self.profile.keyword_casing {
                    KeywordCasing::Upper => format!("{k:?}").to_ascii_uppercase(),
                    KeywordCasing::Lower => format!("{k:?}").to_ascii_lowercase(),
                    KeywordCasing::Preserve => t.text.clone(),
                },
                TokenKind::StringLit(s) => format!("'{}'", s.replace("'", "''")),
                TokenKind::Identifier(s) => s.clone(),
                TokenKind::Integer(n) => n.to_string(),
                TokenKind::Float(n) => n.to_string(),
                TokenKind::HexLiteral(s) => format!("X'{s}'"),
                TokenKind::BinaryLiteral(s) => format!("B'{s}'"),
                TokenKind::Op(op) => format_op(*op),
                TokenKind::LParen => "(".to_string(),
                TokenKind::RParen => ")".to_string(),
                TokenKind::Comma => ",".to_string(),
                TokenKind::Semicolon => ";".to_string(),
                TokenKind::Newline => "\n".to_string(),
                TokenKind::Continuation => "\\\n".to_string(),
                TokenKind::LineComment(c) | TokenKind::BlockComment(c) => c.clone(),
                _ => t.text.clone(),
            };
            parts.push(part);
        }
        parts.join(" ")
    }
}

/// Post-format normalization: remove trailing whitespace and collapse consecutive
/// blank lines to at most `max_blank_lines`.
fn normalize_output(text: &str, max_blank_lines: usize) -> String {
    let mut result: Vec<&str> = Vec::new();
    let mut consecutive_blank = 0usize;

    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank <= max_blank_lines {
                result.push("");
            }
        } else {
            consecutive_blank = 0;
            result.push(trimmed);
        }
    }

    // Drop trailing blank lines; always end with a single newline.
    while result.last() == Some(&"") {
        result.pop();
    }

    if result.is_empty() {
        String::new()
    } else {
        result.join("\n") + "\n"
    }
}

fn format_op(op: rexx_lexer::Op) -> String {
    use rexx_lexer::Op::*;
    match op {
        Plus => "+".to_string(),
        Minus => "-".to_string(),
        Star => "*".to_string(),
        Slash => "/".to_string(),
        SlashSlash => "//".to_string(),
        Percent => "%".to_string(),
        StarStar => "**".to_string(),
        Eq => "=".to_string(),
        NotEq => "\\=".to_string(),
        Lt => "<".to_string(),
        Gt => ">".to_string(),
        Le => "<=".to_string(),
        Ge => ">=".to_string(),
        StrictEq => "==".to_string(),
        StrictNotEq => "\\==".to_string(),
        StrictLt => "<<".to_string(),
        StrictGt => ">>".to_string(),
        StrictLe => "<<=".to_string(),
        StrictGe => ">>=".to_string(),
        Concat => "||".to_string(),
        ConcatBlank => " ".to_string(),
        Assign => ":=".to_string(),
        Not => "\\".to_string(),
        And => "&".to_string(),
        Or => "|".to_string(),
        Xor => "&&".to_string(),
        Colon => ":".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{format_rexx, format_rexx_with_profile, normalize_output};
    use rexx_config::{KeywordCasing, mainframe_compatible, standard};

    #[test]
    fn inserts_first_line_comment() {
        let out = format_rexx("say 'x'\n");
        assert!(out.starts_with("/* The first line of a REXX exec must always be a comment. */"));
    }

    #[test]
    fn formats_do_block() {
        let src = "/* ok */\ndo\nsay 'hi'\nend";
        let out = format_rexx(src);
        assert!(out.contains("    SAY 'hi'"));
    }

    #[test]
    fn mainframe_profile_uppercases_keywords() {
        let src = "/* ok */\nsay 'hi'\ndo\nend";
        let out = format_rexx_with_profile(src, mainframe_compatible());
        assert!(out.contains("SAY 'hi'"));
        assert!(out.contains("DO"));
        assert!(out.contains("END"));
    }

    #[test]
    fn standard_profile_lowercases_keywords() {
        let src = "/* ok */\nSAY 'hi'\nDO\nEND";
        let out = format_rexx_with_profile(src, standard());
        assert!(out.contains("say 'hi'"));
        assert!(out.contains("do"));
        assert!(out.contains("end"));
    }

    #[test]
    fn preserve_casing_keeps_original() {
        let src = "/* ok */\nSay 'hi'\n";
        let mut profile = mainframe_compatible();
        profile.keyword_casing = KeywordCasing::Preserve;
        let out = format_rexx_with_profile(src, profile);
        // Preserve mode: keyword token text is emitted as-is.
        assert!(out.contains("'hi'"));
    }

    #[test]
    fn idempotent_formatting() {
        let src = "/* ok */\nsay 'hi'\ndo\nsay 'world'\nend\n";
        let once = format_rexx(src);
        let twice = format_rexx(&once);
        assert_eq!(once, twice, "format must be idempotent");
    }

    #[test]
    fn normalize_removes_trailing_whitespace() {
        let text = "/* ok */  \nSAY 'hi'   \n";
        let result = normalize_output(text, 1);
        assert!(result.contains("/* ok */\n"));
        assert!(result.contains("SAY 'hi'\n"));
    }

    #[test]
    fn normalize_collapses_blank_lines() {
        let text = "/* ok */\n\n\n\nSAY 'hi'\n";
        let result = normalize_output(text, 1);
        assert!(
            !result.contains("\n\n\n"),
            "too many blank lines: {result:?}"
        );
    }

    #[test]
    fn normalize_single_trailing_newline() {
        let text = "/* ok */\nSAY 'hi'\n\n\n";
        let result = normalize_output(text, 1);
        assert!(result.ends_with("SAY 'hi'\n"), "got: {result:?}");
    }

    #[test]
    fn empty_input_inserts_comment() {
        let out = format_rexx("");
        assert!(out.starts_with("/*"));
    }

    #[test]
    fn respects_indent_size() {
        let src = "/* ok */\ndo\nsay 'hi'\nend\n";
        let mut profile = mainframe_compatible();
        profile.indent_size = 2;
        let out = format_rexx_with_profile(src, profile);
        assert!(
            out.contains("  SAY 'hi'"),
            "expected 2-space indent, got:\n{out}"
        );
    }
}
