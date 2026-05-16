#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Do,
    End,
    If,
    Then,
    Else,
    When,
    Otherwise,
    Select,
    Call,
    Return,
    Exit,
    Signal,
    Say,
    Parse,
    Pull,
    Push,
    Upper,
    Lower,
    Drop,
    Procedure,
    Expose,
    Arg,
    Interpret,
    Nop,
    Address,
    Numeric,
    Trace,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Plus,
    Minus,
    Star,
    Slash,
    SlashSlash,
    Percent,
    StarStar,
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    StrictEq,
    StrictNotEq,
    StrictLt,
    StrictGt,
    StrictLe,
    StrictGe,
    Concat,
    ConcatBlank,
    Assign,
    Not,
    And,
    Or,
    Xor,
    Colon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LineComment(String),
    BlockComment(String),
    StringLit(String),
    Integer(i64),
    Float(f64),
    HexLiteral(String),
    BinaryLiteral(String),
    Identifier(String),
    Label(String),
    Keyword(Keyword),
    Op(Op),
    Newline,
    Comma,
    Semicolon,
    LParen,
    RParen,
    Continuation,
    UnterminatedBlockComment(String),
    Unknown(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: u32,
    pub col: u32,
}

pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    line: u32,
    col: u32,
    prev_can_concat_blank: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            col: 1,
            prev_can_concat_blank: false,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn peek_nth_char(&self, n: usize) -> Option<char> {
        self.remaining().chars().nth(n)
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn token(&self, line: u32, col: u32, kind: TokenKind, text: String) -> Token {
        Token {
            kind,
            text,
            line,
            col,
        }
    }

    fn consume_spaces(&mut self) -> Option<(u32, u32)> {
        let mut start = None;
        while matches!(self.peek_char(), Some(' ' | '\t')) {
            start.get_or_insert((self.line, self.col));
            let _ = self.advance_char();
        }
        start
    }

    fn consume_newline(&mut self) {
        match self.peek_char() {
            Some('\r') => {
                self.pos += '\r'.len_utf8();
                if self.peek_char() == Some('\n') {
                    self.pos += '\n'.len_utf8();
                }
            }
            Some('\n') => {
                self.pos += '\n'.len_utf8();
            }
            _ => return,
        }
        self.line += 1;
        self.col = 1;
    }

    fn starts_with(&self, text: &str) -> bool {
        self.remaining().starts_with(text)
    }

    fn is_trailing_continuation(&self, consumed_len: usize) -> bool {
        let mut chars = self.remaining().chars();
        for _ in 0..consumed_len {
            let _ = chars.next();
        }

        loop {
            match chars.next() {
                Some(' ' | '\t') => continue,
                Some('\r' | '\n') | None => return true,
                _ => return false,
            }
        }
    }

    fn next_starts_concat_term(&self) -> bool {
        match self.peek_char() {
            Some(ch) if ch.is_ascii_digit() || matches!(ch, '\'' | '"') => true,
            Some('x' | 'X' | 'b' | 'B') if self.peek_nth_char(1) == Some('\'') => true,
            Some(ch) if is_ident_start(ch) => {
                let ident: String = self
                    .remaining()
                    .chars()
                    .take_while(|&current| is_ident_continue(current))
                    .collect();
                Keyword::from_ident(&ident).is_none()
            }
            _ => false,
        }
    }

    fn read_block_comment(&mut self) -> TokenKind {
        let _ = self.advance_char(); // /
        let _ = self.advance_char(); // *

        let mut text = String::new();
        let mut depth = 1;

        while let Some(ch) = self.peek_char() {
            if ch == '/' && self.peek_nth_char(1) == Some('*') {
                depth += 1;
                text.push('/');
                text.push('*');
                let _ = self.advance_char();
                let _ = self.advance_char();
                continue;
            }
            if ch == '*' && self.peek_nth_char(1) == Some('/') {
                depth -= 1;
                if depth == 0 {
                    let _ = self.advance_char();
                    let _ = self.advance_char();
                    return TokenKind::BlockComment(text);
                }
                text.push('*');
                text.push('/');
                let _ = self.advance_char();
                let _ = self.advance_char();
                continue;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        TokenKind::UnterminatedBlockComment(text)
    }

    fn read_line_comment(&mut self) -> TokenKind {
        let _ = self.advance_char();
        let _ = self.advance_char();

        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '\r' | '\n') {
                break;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        TokenKind::LineComment(text)
    }

    fn read_string(&mut self, quote: char) -> TokenKind {
        let _ = self.advance_char();

        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            let _ = self.advance_char();
            if ch == quote {
                if self.peek_char() == Some(quote) {
                    let _ = self.advance_char();
                    text.push(quote);
                    continue;
                }
                break;
            }
            text.push(ch);
        }

        TokenKind::StringLit(text)
    }

    fn read_base_literal(&mut self, base: LiteralBase) -> TokenKind {
        let _ = self.advance_char();
        let _ = self.advance_char();

        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '\'' {
                let _ = self.advance_char();
                break;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        match base {
            LiteralBase::Hex => TokenKind::HexLiteral(text),
            LiteralBase::Binary => TokenKind::BinaryLiteral(text),
        }
    }

    fn read_zero_x_literal(&mut self) -> TokenKind {
        let _ = self.advance_char();
        let _ = self.advance_char();

        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_hexdigit() {
                break;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        TokenKind::HexLiteral(text)
    }

    fn read_number(&mut self) -> TokenKind {
        let mut text = String::new();
        let mut is_float = false;

        if self.peek_char() == Some('.') {
            is_float = true;
            text.push('.');
            let _ = self.advance_char();
        }

        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_digit() {
                break;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        if !is_float
            && self.peek_char() == Some('.')
            && matches!(self.peek_nth_char(1), Some(ch) if ch.is_ascii_digit())
        {
            is_float = true;
            text.push('.');
            let _ = self.advance_char();

            while let Some(ch) = self.peek_char() {
                if !ch.is_ascii_digit() {
                    break;
                }
                text.push(ch);
                let _ = self.advance_char();
            }
        }

        if matches!(self.peek_char(), Some('e' | 'E')) {
            let mut ahead = 1;
            let mut has_exponent_digits = false;
            if matches!(self.peek_nth_char(ahead), Some('+' | '-')) {
                ahead += 1;
            }
            if matches!(self.peek_nth_char(ahead), Some(ch) if ch.is_ascii_digit()) {
                has_exponent_digits = true;
            }

            if has_exponent_digits {
                is_float = true;
                text.push(self.advance_char().unwrap()); // e or E
                if matches!(self.peek_char(), Some('+' | '-')) {
                    text.push(self.advance_char().unwrap());
                }
                while let Some(ch) = self.peek_char() {
                    if !ch.is_ascii_digit() {
                        break;
                    }
                    text.push(ch);
                    let _ = self.advance_char();
                }
            }
        }

        if is_float {
            TokenKind::Float(text.parse::<f64>().unwrap_or(0.0))
        } else {
            TokenKind::Integer(text.parse::<i64>().unwrap_or(0))
        }
    }

    fn read_identifier_or_keyword(&mut self) -> TokenKind {
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !is_ident_continue(ch) {
                break;
            }
            text.push(ch);
            let _ = self.advance_char();
        }

        if self.peek_char() == Some(':') {
            let _ = self.advance_char();
            return TokenKind::Label(text);
        }

        if let Some(keyword) = Keyword::from_ident(&text) {
            TokenKind::Keyword(keyword)
        } else {
            TokenKind::Identifier(text)
        }
    }

    fn read_operator(&mut self) -> Option<TokenKind> {
        const MULTI_CHAR_OPS: [(&str, Op); 22] = [
            ("<<=", Op::StrictLe),
            (">>=", Op::StrictGe),
            ("\\==", Op::StrictNotEq),
            ("¬==", Op::StrictNotEq),
            ("//", Op::SlashSlash),
            ("**", Op::StarStar),
            ("\\=", Op::NotEq),
            ("¬=", Op::NotEq),
            ("<>", Op::NotEq),
            ("><", Op::NotEq),
            ("<\\", Op::Le),
            (">\\", Op::Ge),
            ("<¬", Op::Le),
            (">¬", Op::Ge),
            ("<=", Op::Le),
            (">=", Op::Ge),
            ("==", Op::StrictEq),
            ("<<", Op::StrictLt),
            (">>", Op::StrictGt),
            ("||", Op::Concat),
            ("&&", Op::Xor),
            (":=", Op::Assign),
        ];

        for (text, op) in MULTI_CHAR_OPS {
            if self.starts_with(text) {
                for _ in 0..text.chars().count() {
                    let _ = self.advance_char();
                }
                return Some(TokenKind::Op(op));
            }
        }

        let ch = self.peek_char()?;
        let op = match ch {
            '+' => Op::Plus,
            '-' => Op::Minus,
            '*' => Op::Star,
            '/' => Op::Slash,
            '%' => Op::Percent,
            '=' => Op::Eq,
            '<' => Op::Lt,
            '>' => Op::Gt,
            '\\' | '¬' => Op::Not,
            '|' => Op::Or,
            '&' => Op::And,
            ':' => Op::Colon,
            _ => return None,
        };
        let _ = self.advance_char();
        Some(TokenKind::Op(op))
    }

    fn update_concat_state(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::StringLit(_)
                | TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::HexLiteral(_)
                | TokenKind::BinaryLiteral(_)
                | TokenKind::Identifier(_)
                | TokenKind::RParen
        )
    }
}

enum LiteralBase {
    Hex,
    Binary,
}

impl Keyword {
    fn from_ident(text: &str) -> Option<Self> {
        match text.to_ascii_uppercase().as_str() {
            "DO" => Some(Self::Do),
            "END" => Some(Self::End),
            "IF" => Some(Self::If),
            "THEN" => Some(Self::Then),
            "ELSE" => Some(Self::Else),
            "WHEN" => Some(Self::When),
            "OTHERWISE" => Some(Self::Otherwise),
            "SELECT" => Some(Self::Select),
            "CALL" => Some(Self::Call),
            "RETURN" => Some(Self::Return),
            "EXIT" => Some(Self::Exit),
            "SIGNAL" => Some(Self::Signal),
            "SAY" => Some(Self::Say),
            "PARSE" => Some(Self::Parse),
            "PULL" => Some(Self::Pull),
            "PUSH" => Some(Self::Push),
            "UPPER" => Some(Self::Upper),
            "LOWER" => Some(Self::Lower),
            "DROP" => Some(Self::Drop),
            "PROCEDURE" => Some(Self::Procedure),
            "EXPOSE" => Some(Self::Expose),
            "ARG" => Some(Self::Arg),
            "INTERPRET" => Some(Self::Interpret),
            "NOP" => Some(Self::Nop),
            "ADDRESS" => Some(Self::Address),
            "NUMERIC" => Some(Self::Numeric),
            "TRACE" => Some(Self::Trace),
            "OPTIONS" => Some(Self::Options),
            _ => None,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start_pos = self.pos;
            let ch = self.peek_char()?;

            if matches!(ch, ' ' | '\t') {
                let start = self.consume_spaces();
                if self.prev_can_concat_blank && self.next_starts_concat_term() {
                    let (line, col) = start.expect("space start exists when spaces were consumed");
                    self.prev_can_concat_blank = false;
                    let text = self.src[start_pos..self.pos].to_string();
                    return Some(self.token(line, col, TokenKind::Op(Op::ConcatBlank), text));
                }
                continue;
            }

            let line = self.line;
            let col = self.col;

            let kind = match ch {
                '\r' | '\n' => {
                    self.consume_newline();
                    TokenKind::Newline
                }
                '/' if self.peek_nth_char(1) == Some('*') => self.read_block_comment(),
                '-' if self.peek_nth_char(1) == Some('-') => self.read_line_comment(),
                '\'' | '"' => self.read_string(ch),
                '0' if matches!(self.peek_nth_char(1), Some('x' | 'X'))
                    && matches!(self.peek_nth_char(2), Some(next) if next.is_ascii_hexdigit()) =>
                {
                    self.read_zero_x_literal()
                }
                'x' | 'X' if self.peek_nth_char(1) == Some('\'') => {
                    self.read_base_literal(LiteralBase::Hex)
                }
                'b' | 'B' if self.peek_nth_char(1) == Some('\'') => {
                    self.read_base_literal(LiteralBase::Binary)
                }
                ch if ch.is_ascii_digit() => self.read_number(),
                '.' if matches!(self.peek_nth_char(1), Some(next) if next.is_ascii_digit()) => {
                    self.read_number()
                }
                ch if is_ident_start(ch) => self.read_identifier_or_keyword(),
                ',' => {
                    let continuation = self.is_trailing_continuation(1);
                    let _ = self.advance_char();
                    if continuation {
                        TokenKind::Continuation
                    } else {
                        TokenKind::Comma
                    }
                }
                ';' => {
                    let continuation = self.is_trailing_continuation(1);
                    let _ = self.advance_char();
                    if continuation {
                        TokenKind::Continuation
                    } else {
                        TokenKind::Semicolon
                    }
                }
                '(' => {
                    let _ = self.advance_char();
                    TokenKind::LParen
                }
                ')' => {
                    let _ = self.advance_char();
                    TokenKind::RParen
                }
                _ => {
                    if let Some(kind) = self.read_operator() {
                        kind
                    } else {
                        let _ = self.advance_char();
                        TokenKind::Unknown(ch)
                    }
                }
            };

            let text = self.src[start_pos..self.pos].to_string();
            self.prev_can_concat_blank = Self::update_concat_state(&kind);
            return Some(self.token(line, col, kind, text));
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '!' | '?' | '_' | '.')
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '!' | '?' | '_' | '.')
}

#[cfg(test)]
mod tests {
    use super::{Keyword, Lexer, Op, Token, TokenKind};
    use insta::assert_debug_snapshot;

    fn lex(src: &str) -> Vec<Token> {
        Lexer::new(src).collect()
    }

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).into_iter().map(|token| token.kind).collect()
    }

    #[test]
    fn lexes_nested_block_comments() {
        assert_eq!(
            kinds("/* outer /* inner */ outer */"),
            vec![TokenKind::BlockComment(
                " outer /* inner */ outer ".to_string()
            )]
        );
    }

    #[test]
    fn lexes_scientific_notation() {
        assert_eq!(
            kinds("1.2e3, 1E-2, .5E+1"),
            vec![
                TokenKind::Float(1200.0),
                TokenKind::Comma,
                TokenKind::Float(0.01),
                TokenKind::Comma,
                TokenKind::Float(5.0),
            ]
        );
    }

    #[test]
    fn lexes_dot_identifiers() {
        assert_eq!(
            kinds(".ident !ident ?ident"),
            vec![
                TokenKind::Identifier(".ident".to_string()),
                TokenKind::Op(Op::ConcatBlank),
                TokenKind::Identifier("!ident".to_string()),
                TokenKind::Op(Op::ConcatBlank),
                TokenKind::Identifier("?ident".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_all_operators() {
        assert_eq!(
            kinds("<>,\\==,¬==,¬=,><,<\\,>\\,<¬,>¬,<<=,>>=,:=,:"),
            vec![
                TokenKind::Op(Op::NotEq),
                TokenKind::Comma,
                TokenKind::Op(Op::StrictNotEq),
                TokenKind::Comma,
                TokenKind::Op(Op::StrictNotEq),
                TokenKind::Comma,
                TokenKind::Op(Op::NotEq),
                TokenKind::Comma,
                TokenKind::Op(Op::NotEq),
                TokenKind::Comma,
                TokenKind::Op(Op::Le),
                TokenKind::Comma,
                TokenKind::Op(Op::Ge),
                TokenKind::Comma,
                TokenKind::Op(Op::Le),
                TokenKind::Comma,
                TokenKind::Op(Op::Ge),
                TokenKind::Comma,
                TokenKind::Op(Op::StrictLe),
                TokenKind::Comma,
                TokenKind::Op(Op::StrictGe),
                TokenKind::Comma,
                TokenKind::Op(Op::Assign),
                TokenKind::Comma,
                TokenKind::Op(Op::Colon),
            ]
        );
    }

    #[test]
    fn lexes_strings_with_doubled_quote_escape() {
        assert_eq!(
            kinds("'it''s'\n\"a\"\"b\""),
            vec![
                TokenKind::StringLit("it's".to_string()),
                TokenKind::Newline,
                TokenKind::StringLit("a\"b".to_string()),
            ]
        );
    }

    #[test]
    fn lexes_numeric_and_base_literals() {
        assert_eq!(
            kinds("123,3.5,0x1A,X'FF',B'1010'"),
            vec![
                TokenKind::Integer(123),
                TokenKind::Comma,
                TokenKind::Float(3.5),
                TokenKind::Comma,
                TokenKind::HexLiteral("1A".to_string()),
                TokenKind::Comma,
                TokenKind::HexLiteral("FF".to_string()),
                TokenKind::Comma,
                TokenKind::BinaryLiteral("1010".to_string()),
            ]
        );
    }

    #[test]
    fn lexes_keywords_case_insensitively_and_preserves_identifiers() {
        assert_eq!(
            kinds("Do doodle"),
            vec![
                TokenKind::Keyword(Keyword::Do),
                TokenKind::Identifier("doodle".to_string()),
            ]
        );
    }

    #[test]
    fn detects_labels_without_space_before_colon() {
        assert_eq!(
            kinds("start:\nnext :"),
            vec![
                TokenKind::Label("start".to_string()),
                TokenKind::Newline,
                TokenKind::Identifier("next".to_string()),
                TokenKind::Op(Op::Colon),
            ]
        );
    }

    #[test]
    fn snapshot_mixed_snippet() {
        let src = "/* header */\nmain: do\n  say 'It''s' || \"OK\"\n  if count == 2.5e-1 then return;\nend\n-- done";
        assert_debug_snapshot!(lex(src));
    }

    #[test]
    fn tracks_line_and_column_across_newlines() {
        let tokens = lex("a\r\nb\ncc");
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Identifier("a".to_string()),
                    text: "a".to_string(),
                    line: 1,
                    col: 1,
                },
                Token {
                    kind: TokenKind::Newline,
                    text: "\r\n".to_string(),
                    line: 1,
                    col: 2,
                },
                Token {
                    kind: TokenKind::Identifier("b".to_string()),
                    text: "b".to_string(),
                    line: 2,
                    col: 1,
                },
                Token {
                    kind: TokenKind::Newline,
                    text: "\r\n".to_string(),
                    line: 2,
                    col: 2,
                },
                Token {
                    kind: TokenKind::Identifier("cc".to_string()),
                    text: "cc".to_string(),
                    line: 3,
                    col: 1,
                },
            ]
        );
    }

    #[test]
    fn emits_blank_concatenation() {
        assert_eq!(
            kinds("say 'a' name"),
            vec![
                TokenKind::Keyword(Keyword::Say),
                TokenKind::StringLit("a".to_string()),
                TokenKind::Op(Op::ConcatBlank),
                TokenKind::Identifier("name".to_string()),
            ]
        );
    }
}
