use std::iter::Peekable;

use log::trace;

use crate::{
    error::ParserError,
    lexer::LexerIterator,
    token::{Span, Token, TokenType},
    value::Value,
    vm::{Chunk, Loc, OpCode},
};

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn from_u8(n: u8) -> Self {
        trace!("parser::Precedence::from_u8(n: {n})");
        match n {
            0 => Self::None,
            1 => Self::Assignment,
            2 => Self::Or,
            3 => Self::And,
            4 => Self::Equality,
            5 => Self::Comparison,
            6 => Self::Term,
            7 => Self::Factor,
            8 => Self::Unary,
            9 => Self::Call,
            10 => Self::Primary,
            _ => Self::None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ParseFn {
    None,
    Number,
    Group,
    Unary,
    Binary,
    Literal,
    String,
    Variable,
}

fn get_prefix_rule(token_type: TokenType) -> ParseFn {
    trace!("parser::prefix_rule(token_type: {:?})", token_type);
    match token_type {
        TokenType::LeftParen => ParseFn::Group,
        TokenType::RightParen => ParseFn::None,
        TokenType::LeftBrace => ParseFn::None,
        TokenType::RightBrace => ParseFn::None,
        TokenType::Semi => ParseFn::None,
        TokenType::Comma => ParseFn::None,
        TokenType::Dot => ParseFn::None,
        TokenType::Minus => ParseFn::Unary,
        TokenType::Plus => ParseFn::None,
        TokenType::Star => ParseFn::None,
        TokenType::Slash => ParseFn::None,
        TokenType::Equal => ParseFn::None,
        TokenType::EqualEqual => ParseFn::None,
        TokenType::Bang => ParseFn::Unary,
        TokenType::BangEqual => ParseFn::None,
        TokenType::Greater => ParseFn::None,
        TokenType::GreaterEqual => ParseFn::None,
        TokenType::Less => ParseFn::None,
        TokenType::LessEqual => ParseFn::None,
        TokenType::Identifier => ParseFn::Variable,
        TokenType::String => ParseFn::String,
        TokenType::Number => ParseFn::Number,
        TokenType::And => ParseFn::None,
        TokenType::Class => ParseFn::None,
        TokenType::Else => ParseFn::None,
        TokenType::False => ParseFn::Literal,
        TokenType::For => ParseFn::None,
        TokenType::Fun => ParseFn::None,
        TokenType::If => ParseFn::None,
        TokenType::Nil => ParseFn::Literal,
        TokenType::Or => ParseFn::None,
        TokenType::Print => ParseFn::None,
        TokenType::Return => ParseFn::None,
        TokenType::Super => ParseFn::None,
        TokenType::This => ParseFn::None,
        TokenType::True => ParseFn::Literal,
        TokenType::Var => ParseFn::None,
        TokenType::While => ParseFn::None,
        TokenType::Eof => ParseFn::None,
        TokenType::Error => ParseFn::None,
        TokenType::Comment => ParseFn::None,
    }
}

fn get_infix_rule(token_type: TokenType) -> ParseFn {
    trace!("parser::infix_rule(token_type: {:?})", token_type);
    match token_type {
        TokenType::LeftParen => ParseFn::None,
        TokenType::RightParen => ParseFn::None,
        TokenType::LeftBrace => ParseFn::None,
        TokenType::RightBrace => ParseFn::None,
        TokenType::Semi => ParseFn::None,
        TokenType::Comma => ParseFn::None,
        TokenType::Dot => ParseFn::None,
        TokenType::Minus => ParseFn::Binary,
        TokenType::Plus => ParseFn::Binary,
        TokenType::Star => ParseFn::Binary,
        TokenType::Slash => ParseFn::Binary,
        TokenType::Equal => ParseFn::None,
        TokenType::EqualEqual => ParseFn::Binary,
        TokenType::Bang => ParseFn::None,
        TokenType::BangEqual => ParseFn::Binary,
        TokenType::Greater => ParseFn::Binary,
        TokenType::GreaterEqual => ParseFn::Binary,
        TokenType::Less => ParseFn::Binary,
        TokenType::LessEqual => ParseFn::Binary,
        TokenType::Identifier => ParseFn::None,
        TokenType::String => ParseFn::None,
        TokenType::Number => ParseFn::None,
        TokenType::And => ParseFn::None,
        TokenType::Class => ParseFn::None,
        TokenType::Else => ParseFn::None,
        TokenType::False => ParseFn::None,
        TokenType::For => ParseFn::None,
        TokenType::Fun => ParseFn::None,
        TokenType::If => ParseFn::None,
        TokenType::Nil => ParseFn::None,
        TokenType::Or => ParseFn::None,
        TokenType::Print => ParseFn::None,
        TokenType::Return => ParseFn::None,
        TokenType::Super => ParseFn::None,
        TokenType::This => ParseFn::None,
        TokenType::True => ParseFn::None,
        TokenType::Var => ParseFn::None,
        TokenType::While => ParseFn::None,
        TokenType::Eof => ParseFn::None,
        TokenType::Error => ParseFn::None,
        TokenType::Comment => ParseFn::None,
    }
}

fn get_precedence_rule(token_type: TokenType) -> Precedence {
    trace!("parser::precedence_rule(token_type: {:?})", token_type);
    match token_type {
        TokenType::LeftParen => Precedence::None,
        TokenType::RightParen => Precedence::None,
        TokenType::LeftBrace => Precedence::None,
        TokenType::RightBrace => Precedence::None,
        TokenType::Semi => Precedence::None,
        TokenType::Comma => Precedence::None,
        TokenType::Dot => Precedence::None,
        TokenType::Minus => Precedence::Term,
        TokenType::Plus => Precedence::Term,
        TokenType::Star => Precedence::Factor,
        TokenType::Slash => Precedence::Factor,
        TokenType::Equal => Precedence::None,
        TokenType::EqualEqual => Precedence::Equality,
        TokenType::Bang => Precedence::None,
        TokenType::BangEqual => Precedence::Equality,
        TokenType::Greater => Precedence::Comparison,
        TokenType::GreaterEqual => Precedence::Comparison,
        TokenType::Less => Precedence::Comparison,
        TokenType::LessEqual => Precedence::Comparison,
        TokenType::Identifier => Precedence::None,
        TokenType::String => Precedence::None,
        TokenType::Number => Precedence::None,
        TokenType::And => Precedence::None,
        TokenType::Class => Precedence::None,
        TokenType::Else => Precedence::None,
        TokenType::False => Precedence::None,
        TokenType::For => Precedence::None,
        TokenType::Fun => Precedence::None,
        TokenType::If => Precedence::None,
        TokenType::Nil => Precedence::None,
        TokenType::Or => Precedence::None,
        TokenType::Print => Precedence::None,
        TokenType::Return => Precedence::None,
        TokenType::Super => Precedence::None,
        TokenType::This => Precedence::None,
        TokenType::True => Precedence::None,
        TokenType::Var => Precedence::None,
        TokenType::While => Precedence::None,
        TokenType::Eof => Precedence::None,
        TokenType::Error => Precedence::None,
        TokenType::Comment => Precedence::None,
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Peekable<LexerIterator<'a>>,
    previous: Token,
    current: Token,
    had_error: bool,
    panic: bool,
    chunk: &'a mut Chunk,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Peekable<LexerIterator<'a>>, chunk: &'a mut Chunk) -> Self {
        trace!("parser::Parser::new(source, tokens, chunk: {:?})", chunk);
        Self {
            source,
            tokens,
            previous: Token::new(TokenType::Error, 0, 0, Span::new(0, 0), None),
            current: Token::new(TokenType::Error, 0, 0, Span::new(0, 0), None),
            had_error: false,
            panic: false,
            chunk,
        }
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        trace!("parser::Parser::parse()");
        self.advance();

        while !self.match_token(TokenType::Eof) {
            self.declaration();
        }

        Ok(())
    }
}

impl<'a> Parser<'a> {
    fn error_at_current(&mut self, message: &str) {
        self.had_error = true;
        eprint!("[{}:{}] Error", self.current.line, self.current.col);

        if self.current.token_type == TokenType::Eof {
            eprint!(" at end");
        }

        eprintln!(": {message}");
    }
    fn error_at_previous(&mut self, message: &str) {
        self.had_error = true;
        eprint!("[{}:{}] Error", self.previous.line, self.previous.col);

        if self.previous.token_type == TokenType::Eof {
            eprint!(" at end");
        }

        eprintln!(": {message}");
    }

    fn synchronize(&mut self) {
        trace!("parser::Parser::synchronize()");
        self.panic = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semi {
                break;
            }

            match self.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}

impl<'a> Parser<'a> {
    fn advance(&mut self) {
        trace!("parser::Parser::advance()");
        self.previous = self.current.clone();

        loop {
            let Some(maybe_token) = self.tokens.next() else {
                break;
            };

            match maybe_token {
                Ok(token) => {
                    self.current = token;
                    break;
                }
                Err(e) => {
                    e.report();
                    continue;
                }
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, error_message: &str) {
        trace!(
            "parser::Parser::consume(token_type: {:?}, error_message: {error_message})",
            token_type
        );
        if self.current.token_type == token_type {
            self.advance();
        } else {
            self.error_at_current(error_message);
        }
    }

    fn check_type(&self, token_type: TokenType) -> bool {
        trace!("parser::Parser::check_type(token_type: {:?})", token_type);
        self.current.token_type == token_type
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        trace!("parser::Parser::match_token(token_type: {:?})", token_type);
        if !self.check_type(token_type) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn span_to_str(&self, span: Span) -> &str {
        trace!("parser::Parser::span_to_str(span: {:?})", span);
        &self.source[span.start..span.end]
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        trace!(
            "parser::Parser::parse_precedence(precedence: {:?})",
            precedence
        );
        self.advance();

        let prefix_rule = get_prefix_rule(self.previous.token_type.clone());

        if prefix_rule == ParseFn::None {
            self.error_at_current("Expected expression.");
            return;
        }

        let can_assign = precedence <= Precedence::Assignment;
        self.parse_fn(prefix_rule, can_assign);

        while precedence <= get_precedence_rule(self.current.token_type.clone()) {
            self.advance();
            let infix_rule = get_infix_rule(self.previous.token_type.clone());
            self.parse_fn(infix_rule, can_assign);
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error_at_current("Invalid assignment target.");
        }
    }

    fn parse_fn(&mut self, parse_fn: ParseFn, can_assign: bool) {
        trace!("parser::Parser::parse_fn(parse_fn: {:?})", parse_fn);
        match parse_fn {
            ParseFn::Number => self.number(),
            ParseFn::Group => self.group(),
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Literal => self.literal(),
            ParseFn::String => self.string(),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::None => {}
        }
    }

    fn expression(&mut self) {
        trace!("parser::Parser::expression()");
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        trace!("parser::Parser::number()");
        let num_str = self.span_to_str(self.previous.lexeme.clone().unwrap());
        let value = num_str.parse::<f64>().unwrap();
        self.emit_constant(Value::Number { value });
    }

    fn group(&mut self) {
        trace!("parser::Parser::group()");
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after expression.");
    }

    fn unary(&mut self) {
        trace!("parser::Parser::unary()");
        let operator_type = self.previous.token_type.clone();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_op(OpCode::Negate),
            TokenType::Bang => self.emit_op(OpCode::Not),
            _ => return,
        }
    }

    fn binary(&mut self) {
        trace!("parser::Parser::binary()");
        let operator_type = self.previous.token_type.clone();
        let precedence = get_precedence_rule(operator_type.clone());

        self.parse_precedence(Precedence::from_u8(precedence as u8 + 1));

        match operator_type {
            TokenType::Plus => self.emit_op(OpCode::Add),
            TokenType::Minus => self.emit_op(OpCode::Subtract),
            TokenType::Star => self.emit_op(OpCode::Multiply),
            TokenType::Slash => self.emit_op(OpCode::Divide),
            TokenType::EqualEqual => self.emit_op(OpCode::Equal),
            TokenType::BangEqual => self.emit_ops(OpCode::Equal, OpCode::Not),
            TokenType::Greater => self.emit_op(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_ops(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_op(OpCode::Less),
            TokenType::LessEqual => self.emit_ops(OpCode::Greater, OpCode::Not),
            _ => return,
        }
    }

    fn literal(&mut self) {
        trace!("parser::Parser::literal()");

        match self.previous.token_type {
            TokenType::False => self.emit_op(OpCode::False),
            TokenType::True => self.emit_op(OpCode::True),
            TokenType::Nil => self.emit_op(OpCode::Nil),
            _ => {}
        }
    }

    fn string(&mut self) {
        trace!("parser::Parser::string()");
        let lexeme = self.previous.lexeme.clone().unwrap();
        self.emit_constant(Value::String {
            value: String::from(&self.source[lexeme.start..lexeme.end]),
        });
    }

    fn variable(&mut self, can_assign: bool) {
        trace!("parser::Parser::variable()");
        self.named_variable(self.previous.literal.clone(), can_assign);
    }

    fn named_variable(&mut self, name: Span, can_assign: bool) {
        trace!("parser::Parser::named_variable(name: {:?}", name);
        let arg = self.identifier_constant(name);

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_ops_usize(OpCode::SetGlobal, arg);
        } else {
            self.emit_ops_usize(OpCode::GetGlobal, arg);
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        trace!("parser::Parser::parse_variable()");
        self.consume(TokenType::Identifier, error_message);
        self.identifier_constant(self.previous.literal.clone())
    }

    fn identifier_constant(&mut self, name: Span) -> usize {
        trace!("parser::Parser::identifier_constant(name: {:?})", name);
        self.chunk.add_constant(Value::String {
            value: String::from(&self.source[name.start..name.end]),
        })
    }

    fn define_variable(&mut self, global: usize) {
        trace!("parser::Parser::define_variable(global: {global})");
        self.emit_op(OpCode::DefineGlobal);
        self.emit_op_usize(global);
    }

    fn declaration(&mut self) {
        trace!("parser::Parser::declaration()");
        match self.current.token_type {
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }

        if self.panic {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        trace!("parser::Parser::var_declaration()");
        self.advance();

        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Nil);
        }

        self.consume(TokenType::Semi, "Expected ';' after variable declaration");

        self.define_variable(global);
    }

    fn statement(&mut self) {
        trace!("parser::Parser::statement()");
        match self.current.token_type {
            TokenType::Print => self.print_statement(),
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) {
        trace!("parser::Parser::print_statement()");
        self.advance();
        self.expression();
        self.consume(TokenType::Semi, "Expected ';' after value.");
        self.emit_op(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        trace!("parser::Parser::expression_statement()");
        self.expression();
        self.consume(TokenType::Semi, "Expected ';' after expression.");
        self.emit_op(OpCode::Pop);
    }
}

impl<'a> Parser<'a> {
    fn emit_op(&mut self, op: OpCode) {
        trace!("parser::Parser::emit_op(op: {:?})", op);
        self.chunk
            .write(op as usize, Loc::new(self.previous.line, self.previous.col));
    }

    fn emit_op_usize(&mut self, op: usize) {
        trace!("parser::Parser::emit_op(op: {op})");
        self.chunk
            .write(op, Loc::new(self.previous.line, self.previous.col));
    }

    fn emit_ops(&mut self, op1: OpCode, op2: OpCode) {
        trace!("parser::Parser::emit_ops(op1: {:?}, op2: {:?})", op1, op2);
        self.emit_op(op1);
        self.emit_op(op2);
    }

    fn emit_ops_usize(&mut self, op1: OpCode, op2: usize) {
        trace!("parser::Parser::emit_ops_usize(op1: {:?}, op2: {op2})", op1);
        self.emit_op(op1);
        self.emit_op_usize(op2);
    }

    fn emit_constant(&mut self, value: Value) {
        trace!("parser::Parser::emit_constant(value: {value})");
        let pos = self.chunk.add_constant(value);
        self.emit_op(OpCode::Constant);
        self.emit_op_usize(pos);
    }
}
