use rexx_diagnostics::Span;
pub use rexx_lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Label(Label),
    DoBlock(DoBlock),
    SelectBlock(SelectBlock),
    IfStatement(IfStatement),
    Command(Command),
    Comment(Token),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Label(l) => l.span,
            Statement::DoBlock(b) => b.span,
            Statement::SelectBlock(b) => b.span,
            Statement::IfStatement(i) => i.span,
            Statement::Command(c) => c.span,
            Statement::Comment(t) => Span::new(t.line, t.col, t.line, t.col),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoBlock {
    pub header: Command,
    pub body: Vec<Statement>,
    pub footer_tokens: Vec<Token>, // tokens of the END
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectBlock {
    pub header_span: Span,
    pub cases: Vec<WhenCase>,
    pub otherwise: Option<OtherwiseCase>,
    pub footer_tokens: Vec<Token>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenCase {
    pub condition: Command,
    pub body: Box<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtherwiseCase {
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Command,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub tokens: Vec<Token>,
    pub span: Span,
}
