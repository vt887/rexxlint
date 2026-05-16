use rexx_ast::*;
use rexx_config::{ConfigError, FormattingProfile, default_profile, load_profile};
use rexx_lexer::{Token, TokenKind};
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
    let lexer = rexx_lexer::Lexer::new(input);
    let tokens: Vec<Token> = lexer.collect();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse_program();

    let mut formatter = Formatter::new(profile);
    formatter.format(&program)
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
        " ".repeat(self.indent_level * 4)
    }

    fn format(&mut self, prog: &Program) -> String {
        if prog.statements.is_empty() {
            return "/* The first line of a REXX exec must always be a comment. */\n".to_string();
        }

        let mut has_first_comment = false;
        if let Some(Statement::Comment(_)) = prog.statements.first() {
            has_first_comment = true;
        }

        if !has_first_comment {
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
            Statement::Label(l) => {
                self.output.push(format!("{}:", l.name));
            }
            Statement::Comment(t) => {
                let text = match &t.kind {
                    TokenKind::LineComment(s) => format!("--{}", s),
                    TokenKind::BlockComment(s) => format!("/*{}*/", s),
                    TokenKind::UnterminatedBlockComment(s) => format!("/*{}", s),
                    _ => unreachable!(),
                };
                self.output.push(format!("{}{}", self.indent(), text));
            }
            Statement::Command(c) => {
                let line = self.format_command(c);
                if !line.is_empty() {
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
                let select_kw = if self.profile.uppercase_keywords {
                    "SELECT"
                } else {
                    "select"
                };
                self.output.push(format!("{}{}", self.indent(), select_kw));
                self.indent_level += 1;
                for case in &b.cases {
                    let cond = self.format_command(&case.condition);
                    let when = if self.profile.uppercase_keywords {
                        "WHEN"
                    } else {
                        "when"
                    };
                    let then = if self.profile.uppercase_keywords {
                        "THEN"
                    } else {
                        "then"
                    };
                    self.output
                        .push(format!("{}{} {} {}", self.indent(), when, cond, then));
                    self.indent_level += 1;
                    self.format_statement(&case.body);
                    self.indent_level -= 1;
                }
                if let Some(otherwise) = &b.otherwise {
                    let ow = if self.profile.uppercase_keywords {
                        "OTHERWISE"
                    } else {
                        "otherwise"
                    };
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
                let if_kw = if self.profile.uppercase_keywords {
                    "IF"
                } else {
                    "if"
                };
                let then_kw = if self.profile.uppercase_keywords {
                    "THEN"
                } else {
                    "then"
                };
                self.output
                    .push(format!("{}{} {} {}", self.indent(), if_kw, cond, then_kw));
                self.indent_level += 1;
                self.format_statement(&i.then_branch);
                self.indent_level -= 1;
                if let Some(else_branch) = &i.else_branch {
                    let else_kw = if self.profile.uppercase_keywords {
                        "ELSE"
                    } else {
                        "else"
                    };
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
                TokenKind::Keyword(k) => {
                    if self.profile.uppercase_keywords {
                        format!("{:?}", k).to_ascii_uppercase()
                    } else {
                        format!("{:?}", k).to_ascii_lowercase()
                    }
                }
                TokenKind::StringLit(s) => format!("'{}'", s.replace("'", "''")),
                TokenKind::Identifier(s) => s.clone(),
                TokenKind::Integer(n) => n.to_string(),
                TokenKind::Float(n) => n.to_string(),
                TokenKind::HexLiteral(s) => format!("X'{}'", s),
                TokenKind::BinaryLiteral(s) => format!("B'{}'", s),
                TokenKind::Op(op) => format_op(*op),
                TokenKind::LParen => "(".to_string(),
                TokenKind::RParen => ")".to_string(),
                TokenKind::Comma => ",".to_string(),
                TokenKind::Semicolon => ";".to_string(),
                TokenKind::Label(s) => format!("{}:", s),
                _ => "".to_string(),
            };
            if !part.is_empty() {
                parts.push(part);
            }
        }
        parts.join(" ")
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
    use super::{format_rexx, format_rexx_with_profile};
    use rexx_config::mainframe_compatible;

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
}
