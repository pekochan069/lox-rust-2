use std::iter::Peekable;
use std::str::Chars;

use log::trace;

use crate::{
    error::LexerError,
    token::{Span, Token, TokenType},
};

pub struct Lexer<'a> {
    raw_source: &'a str,
    source: Peekable<Chars<'a>>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
    col_start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        trace!("lexer::Lexer::new()");
        Self {
            raw_source: source,
            source: source.chars().peekable(),
            start: 0,
            current: 0,
            line: 1,
            col: 0,
            col_start: 0,
        }
    }

    pub fn iter(&'a mut self) -> LexerIterator<'a> {
        trace!("lexer::Lexer::iter()");
        LexerIterator {
            lexer: self,
            is_eof: false,
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn skip_whitespace(&mut self) {
        trace!("lexer::Lexer::skip_whitespace()");
        while let Some(c) = self.source.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.current += 1;
                    self.col += 1;
                }
                '\n' => {
                    self.current += 1;
                    self.line += 1;
                    self.col = 0;
                }
                _ => break,
            }

            _ = self.source.next();
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::next_token()");
        self.col_start = self.col;
        self.start = self.current;

        let Some(c) = self.advance() else {
            return Ok(self.create_token(TokenType::Eof));
        };

        match c {
            '(' => Ok(self.create_token(TokenType::LeftParen)),
            ')' => Ok(self.create_token(TokenType::RightParen)),
            '{' => Ok(self.create_token(TokenType::LeftBrace)),
            '}' => Ok(self.create_token(TokenType::RightBrace)),
            ';' => Ok(self.create_token(TokenType::Semi)),
            ',' => Ok(self.create_token(TokenType::Comma)),
            '.' => Ok(self.create_token(TokenType::Dot)),
            '+' => Ok(self.create_token(TokenType::Plus)),
            '-' => Ok(self.create_token(TokenType::Minus)),
            '*' => Ok(self.create_token(TokenType::Star)),
            '=' => {
                if self.match_char('=') {
                    Ok(self.create_token(TokenType::EqualEqual))
                } else {
                    Ok(self.create_token(TokenType::Equal))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(self.create_token(TokenType::BangEqual))
                } else {
                    Ok(self.create_token(TokenType::Bang))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(self.create_token(TokenType::GreaterEqual))
                } else {
                    Ok(self.create_token(TokenType::Greater))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(self.create_token(TokenType::LessEqual))
                } else {
                    Ok(self.create_token(TokenType::Less))
                }
            }
            '/' => {
                if self.match_char('/') {
                    self.single_line_comment()
                } else if self.match_char('*') {
                    self.multi_line_comment()
                } else {
                    Ok(self.create_token(TokenType::Slash))
                }
            }
            '"' | '\'' => self.string(c),
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(c),
            _ => Err(LexerError::UnexpectedCharacter {
                line: self.line,
                col: self.col + 1,
            }),
        }
    }
}

impl<'a> Lexer<'a> {
    fn create_token(&self, token_type: TokenType) -> Token {
        trace!("lexer::Lexer::create_token");
        Token::new(
            token_type,
            self.line,
            self.col_start,
            Span::new(self.start, self.current),
        )
    }

    fn advance(&mut self) -> Option<char> {
        trace!("lexer::Lexer::advance()");
        if let Some(c) = self.source.next() {
            let len = c.len_utf8();
            self.current += len;
            self.col += len;
            Some(c)
        } else {
            None
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        trace!("lexer::Lexer::match_char(expected_char: {expected})");
        match self.source.peek().copied() {
            Some(c) => {
                if c == expected {
                    _ = self.source.next();
                    let len = c.len_utf8();
                    self.current += len;
                    self.col += len;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn next_whitespace(&mut self) {
        trace!("lexer::Lexer::next_whitespace()");
        loop {
            let Some(c) = self.source.peek().copied() else {
                break;
            };

            match c {
                ' ' | '\r' | '\t' | '\n' => {
                    break;
                }
                _ => {
                    self.current += c.len_utf8();
                    self.col += c.len_utf8();
                    _ = self.source.next();
                }
            }
        }
    }

    fn check_keyword(&mut self, start: usize, end: usize, snippet: &str) -> bool {
        trace!("lexer::Lexer::check_keyword(start: {start}, end: {end}, snippet: {snippet})");
        if end <= self.raw_source.len() {
            if &self.raw_source[start..end] == snippet {
                for _ in start..end {
                    _ = self.source.next();
                }
                self.current += end - start;
                self.col += end - start;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn single_line_comment(&mut self) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::single_line_comment()");
        loop {
            let Some(next_c) = self.source.peek().copied() else {
                break;
            };

            _ = self.source.next();

            if next_c == '\n' {
                break;
            } else {
                self.current += 1;
                self.col += 1;
            }
        }

        let token = Ok(self.create_token(TokenType::Comment));

        self.line += 1;
        self.current += 1;
        self.col = 0;

        token
    }

    fn multi_line_comment(&mut self) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::multi_line_comment()");
        let start_line = self.line;

        loop {
            let Some(next_c) = self.source.peek().copied() else {
                let error = Err(LexerError::CommentNotTerminated {
                    line: self.line,
                    col: self.col + 2,
                });
                self.next_whitespace();
                return error;
            };

            _ = self.source.next();

            if next_c == '*' {
                let Some(next_next_c) = self.source.peek().copied() else {
                    let error = Err(LexerError::UnterminatedString {
                        line: self.line,
                        col: self.col + 2,
                    });
                    self.next_whitespace();
                    return error;
                };

                if next_next_c == '/' {
                    _ = self.source.next();
                    self.current += 2;
                    self.col += 2;
                    break;
                }
            } else if next_c == '\n' {
                self.current += 1;
                self.col = 0;
                self.line += 1;
            } else {
                self.current += next_c.len_utf8();
                self.col += next_c.len_utf8();
            }
        }

        Ok(Token::new(
            TokenType::Comment,
            start_line,
            self.col_start,
            Span::new(self.start, self.current),
        ))
    }

    fn string(&mut self, c: char) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::string(c: {c})");
        loop {
            let Some(next_c) = self.source.peek().copied() else {
                let error = Err(LexerError::UnterminatedString {
                    line: self.line,
                    col: self.col + 2,
                });
                self.next_whitespace();
                return error;
            };

            _ = self.source.next();

            if next_c == c {
                self.current += 1;
                self.col += 1;
                break;
            } else if next_c == '\n' {
                self.current += next_c.len_utf8();
                self.col = 0;
                self.line += 1;
            } else {
                self.current += next_c.len_utf8();
                self.col += next_c.len_utf8();
            }
        }
        Ok(self.create_token(TokenType::String))
    }

    fn number(&mut self) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::number()");
        let mut has_dot = false;

        loop {
            let Some(next_c) = self.source.peek().copied() else {
                return Ok(self.create_token(TokenType::Number));
            };

            match next_c {
                '.' => {
                    if has_dot {
                        let error = Err(LexerError::UnexpectedCharacter {
                            line: self.line,
                            col: self.col + 2,
                        });
                        self.next_whitespace();
                        return error;
                    }

                    has_dot = true;
                    _ = self.source.next();
                }
                '0'..='9' => {
                    self.current += 1;
                    self.col += 1;
                    _ = self.source.next();
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let error = Err(LexerError::UnexpectedCharacter {
                        line: self.line,
                        col: self.col + 2,
                    });
                    self.next_whitespace();
                    return error;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(self.create_token(TokenType::Number))
    }

    fn identifier(&mut self, c: char) -> Result<Token, LexerError> {
        trace!("lexer::Lexer::identifier(c: {c})");
        match c {
            'a' => {
                if self.check_keyword(self.start + 1, self.start + 3, "and") {
                    return Ok(self.create_token(TokenType::And));
                }
            }
            'c' => {
                if self.check_keyword(self.start + 1, self.start + 5, "lass") {
                    return Ok(self.create_token(TokenType::Class));
                }
            }
            'e' => {
                if self.check_keyword(self.start + 1, self.start + 4, "lse") {
                    return Ok(self.create_token(TokenType::Else));
                }
            }
            'i' => {
                if self.check_keyword(self.start + 1, self.start + 2, "f") {
                    return Ok(self.create_token(TokenType::If));
                }
            }
            'n' => {
                if self.check_keyword(self.start + 1, self.start + 3, "il") {
                    return Ok(self.create_token(TokenType::Nil));
                }
            }
            'o' => {
                if self.check_keyword(self.start + 1, self.start + 2, "r") {
                    return Ok(self.create_token(TokenType::Or));
                }
            }
            'p' => {
                if self.check_keyword(self.start + 1, self.start + 5, "rint") {
                    return Ok(self.create_token(TokenType::Print));
                }
            }
            'r' => {
                if self.check_keyword(self.start + 1, self.start + 6, "eturn") {
                    return Ok(self.create_token(TokenType::Return));
                }
            }
            's' => {
                if self.check_keyword(self.start + 1, self.start + 5, "uper") {
                    return Ok(self.create_token(TokenType::Super));
                }
            }
            'v' => {
                if self.check_keyword(self.start + 1, self.start + 3, "ar") {
                    return Ok(self.create_token(TokenType::Var));
                }
            }
            'w' => {
                if self.check_keyword(self.start + 1, self.start + 5, "hile") {
                    return Ok(self.create_token(TokenType::While));
                }
            }
            'f' => {
                let Some(next_c) = self.source.peek().copied() else {
                    return Ok(self.create_token(TokenType::Identifier));
                };

                if next_c == 'a' {
                    if self.check_keyword(self.start + 2, self.start + 5, "lse") {
                        _ = self.source.next();
                        self.current += 1;
                        self.col += 1;
                        return Ok(self.create_token(TokenType::False));
                    }
                } else if next_c == 'o' {
                    if self.check_keyword(self.start + 2, self.start + 3, "r") {
                        _ = self.source.next();
                        self.current += 1;
                        self.col += 1;
                        return Ok(self.create_token(TokenType::For));
                    }
                } else if next_c == 'u' {
                    if self.check_keyword(self.start + 2, self.start + 3, "n") {
                        _ = self.source.next();
                        self.current += 1;
                        self.col += 1;
                        return Ok(self.create_token(TokenType::Fun));
                    }
                }
            }
            't' => {
                let Some(next_c) = self.source.peek().copied() else {
                    return Ok(self.create_token(TokenType::Identifier));
                };

                if next_c == 'h' {
                    if self.check_keyword(self.start + 2, self.start + 4, "is") {
                        _ = self.source.next();
                        self.current += 1;
                        self.col += 1;
                        return Ok(self.create_token(TokenType::This));
                    }
                } else if next_c == 'r' {
                    if self.check_keyword(self.start + 2, self.start + 4, "ue") {
                        _ = self.source.next();
                        self.current += 1;
                        self.col += 1;
                        return Ok(self.create_token(TokenType::True));
                    }
                }
            }
            _ => {}
        }

        loop {
            let Some(next_c) = self.source.peek().copied() else {
                return Ok(self.create_token(TokenType::Identifier));
            };

            match next_c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    self.current += 1;
                    self.col += 1;
                    _ = self.source.next();
                }
                _ => break,
            }
        }

        Ok(self.create_token(TokenType::Identifier))
    }
}

pub struct LexerIterator<'a> {
    lexer: &'a mut Lexer<'a>,
    is_eof: bool,
}

impl<'a> Iterator for LexerIterator<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        trace!("lexer::LexerIterator::next()");
        if self.is_eof {
            return None;
        }
        self.lexer.skip_whitespace();

        match self.lexer.next_token() {
            Ok(token) => match token.token_type {
                TokenType::Eof => {
                    self.is_eof = true;
                    Some(Ok(token))
                }
                _ => Some(Ok(token)),
            },
            Err(e) => Some(Err(e)),
        }
    }
}
