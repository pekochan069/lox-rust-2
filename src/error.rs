use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LexerError {
    UnexpectedCharacter { line: usize, col: usize },
    UnterminatedString { line: usize, col: usize },
    InvalidNumber { line: usize, col: usize },
    CommentNotTerminated { line: usize, col: usize },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { line: _, col: _ } => {
                write!(f, "Unexpected character")
            }
            Self::UnterminatedString { line: _, col: _ } => {
                write!(f, "Unterminated string")
            }
            Self::InvalidNumber { line: _, col: _ } => {
                write!(f, "Invalid number")
            }
            Self::CommentNotTerminated { line: _, col: _ } => {
                write!(f, "Comment is not terminated")
            }
        }
    }
}

impl Error for LexerError {}

impl LexerError {
    pub fn report(&self) {
        eprintln!("{}", self);
    }
}

#[derive(Debug, Clone)]
pub enum ParserError {
    UnexpectedToken { line: usize, col: usize },
    OutOfSourceBoundary,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { line, col } => write!(f, "[{line}:{col}] Unexpected token"),
            Self::OutOfSourceBoundary => write!(f, "Out of source boundary"),
        }
    }
}

impl Error for ParserError {}

#[derive(Debug, Clone)]
pub enum LoxError {
    Lexer(LexerError),
    Parser(ParserError),
}

impl From<LexerError> for LoxError {
    fn from(err: LexerError) -> Self {
        LoxError::Lexer(err)
    }
}

impl From<ParserError> for LoxError {
    fn from(err: ParserError) -> Self {
        LoxError::Parser(err)
    }
}
