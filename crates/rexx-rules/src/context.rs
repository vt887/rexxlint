#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub lower: String,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct CodeLine {
    pub line_no: usize,
    pub raw: String,
    pub text: String,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct RuleContext {
    pub source: String,
    pub lines: Vec<CodeLine>,
    pub has_unclosed_comment: bool,
}

pub fn build_context(source: &str) -> RuleContext {
    let mut lines = Vec::new();
    let mut in_block_comment = false;

    for (idx, raw) in source.lines().enumerate() {
        let mut cleaned = String::new();
        let mut chars = raw.chars().peekable();
        let mut in_single = false;
        let mut in_double = false;
        while let Some(ch) = chars.next() {
            let next = chars.peek().copied();
            if in_block_comment {
                if ch == '*' && next == Some('/') {
                    in_block_comment = false;
                    let _ = chars.next();
                }
                continue;
            }
            if !in_single && !in_double && ch == '/' && next == Some('*') {
                in_block_comment = true;
                let _ = chars.next();
                continue;
            }
            if !in_double && ch == '\'' {
                // doubled quote inside string is an escape, not string end
                if in_single && chars.peek() == Some(&'\'') {
                    cleaned.push(' ');
                    cleaned.push(' ');
                    let _ = chars.next();
                    continue;
                }
                in_single = !in_single;
                cleaned.push(' ');
                continue;
            }
            if !in_single && ch == '"' {
                if in_double && chars.peek() == Some(&'"') {
                    cleaned.push(' ');
                    cleaned.push(' ');
                    let _ = chars.next();
                    continue;
                }
                in_double = !in_double;
                cleaned.push(' ');
                continue;
            }
            if in_single || in_double {
                cleaned.push(' ');
            } else {
                cleaned.push(ch);
            }
        }

        let mut tokens = Vec::new();
        let bytes = cleaned.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let text = cleaned[start..i].to_string();
                tokens.push(Token {
                    lower: text.to_ascii_lowercase(),
                    text,
                    column: start + 1,
                });
            } else {
                i += 1;
            }
        }

        lines.push(CodeLine {
            line_no: idx + 1,
            raw: raw.to_string(),
            text: cleaned,
            tokens,
        });
    }

    RuleContext {
        source: source.to_string(),
        lines,
        has_unclosed_comment: in_block_comment,
    }
}
