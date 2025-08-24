use log::trace;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Error,
    Eof,
    Comment,
    // Single-Character
    Semi,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Plus,
    Minus,
    Slash,
    Star,
    // Single-or-Multi-Character
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literal
    Identifier,
    String,
    Number,
    // Keyword
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        trace!("token::Span::new(start: {start}, end: {end})");
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub col: usize,
    pub literal: Span,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, col: usize, literal: Span) -> Self {
        trace!(
            "token::Token::new(token_type: {:?}, line: {line}, col: {col}, literal: {:?})",
            token_type, literal
        );
        Self {
            token_type,
            line,
            col,
            literal,
        }
    }

    pub fn len(&self) -> usize {
        self.literal.end - self.literal.start
    }
}
