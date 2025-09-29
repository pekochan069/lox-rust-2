use std::{iter::Peekable, rc::Rc};

use log::trace;

#[cfg(feature = "trace_execution")]
use crate::debug::disassemble_chunk;

use crate::{
    error::ParserError,
    function::{Function, FunctionType},
    lexer::LexerIterator,
    token::{Span, Token, TokenType},
    value::Value,
    vm::{Chunk, OpCode},
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
    And,
    Or,
    Call,
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
        TokenType::LeftParen => ParseFn::Call,
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
        TokenType::And => ParseFn::And,
        TokenType::Class => ParseFn::None,
        TokenType::Else => ParseFn::None,
        TokenType::False => ParseFn::None,
        TokenType::For => ParseFn::None,
        TokenType::Fun => ParseFn::None,
        TokenType::If => ParseFn::None,
        TokenType::Nil => ParseFn::None,
        TokenType::Or => ParseFn::Or,
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
        TokenType::LeftParen => Precedence::Call,
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
        TokenType::And => Precedence::And,
        TokenType::Class => Precedence::None,
        TokenType::Else => Precedence::None,
        TokenType::False => Precedence::None,
        TokenType::For => Precedence::None,
        TokenType::Fun => Precedence::None,
        TokenType::If => Precedence::None,
        TokenType::Nil => Precedence::None,
        TokenType::Or => Precedence::Or,
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

#[derive(Debug)]
pub struct Local {
    name: Span,
    depth: usize,
}

impl Local {
    pub fn new(name: Span, depth: usize) -> Self {
        Self { name, depth }
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }
}

#[derive(Debug)]
pub struct CompileFrame {
    pub function: Function,
    pub function_type: FunctionType,
    pub slots: Vec<Value>,
    pub locals: Vec<Local>,
    pub scope_depth: usize,
}

impl CompileFrame {
    pub fn new(
        function: Function,
        function_type: FunctionType,
        slots: Vec<Value>,
        locals: Vec<Local>,
        scope_depth: usize,
    ) -> Self {
        Self {
            function,
            function_type,
            slots,
            locals,
            scope_depth,
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.locals.clear();
        self.function.chunk.clear();
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Peekable<LexerIterator<'a>>,
    previous: Token,
    current: Token,
    had_error: bool,
    panic: bool,
    frames: Vec<CompileFrame>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Peekable<LexerIterator<'a>>) -> Self {
        trace!("parser::Parser::new(source, tokens)");

        let root_frame = CompileFrame::new(
            Function::new(0, Chunk::new(), None),
            FunctionType::Script,
            vec![],
            vec![],
            0,
        );

        Self {
            source,
            tokens,
            previous: Token::new(TokenType::Error, 0, 0, Span::new(0, 0)),
            current: Token::new(TokenType::Error, 0, 0, Span::new(0, 0)),
            had_error: false,
            panic: false,
            frames: vec![root_frame],
        }
    }

    pub fn parse(&mut self) -> Result<Function, ParserError> {
        trace!("parser::Parser::parse()");
        self.advance();

        while !self.match_token(TokenType::Eof) {
            self.declaration();
        }

        self.end_parse();

        let frame = self.frames.remove(0);
        let function = frame.function;

        Ok(function)
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
    // fn error_at_previous(&mut self, message: &str) {
    //     self.had_error = true;
    //     eprint!("[{}:{}] Error", self.previous.line, self.previous.col);

    //     if self.previous.token_type == TokenType::Eof {
    //         eprint!(" at end");
    //     }

    //     eprintln!(": {message}");
    // }

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
    #[inline]
    fn current_frame(&self) -> &CompileFrame {
        self.frames.last().unwrap()
    }

    #[inline]
    fn current_frame_mut(&mut self) -> &mut CompileFrame {
        self.frames.last_mut().unwrap()
    }

    fn advance(&mut self) {
        trace!("parser::Parser::advance()");
        self.previous = self.current.clone();

        loop {
            let Some(maybe_token) = self.tokens.next() else {
                break;
            };

            match maybe_token {
                Ok(token) => {
                    if token.token_type == TokenType::Comment {
                        continue;
                    }

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

    fn end_parse(&mut self) {
        #[cfg(feature = "trace_execution")]
        {
            if self.had_error {
                let frame = self.current_frame();
                match &frame.function.name {
                    Some(name) => disassemble_chunk(name.as_str(), &frame.function.chunk),
                    None => disassemble_chunk("<script>", &frame.function.chunk),
                }
            }
        }
        self.emit_ops(OpCode::Nil, OpCode::Return);
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
            ParseFn::And => self.and(),
            ParseFn::Or => self.or(),
            ParseFn::Call => self.call(),
            ParseFn::None => {}
        }
    }

    fn expression(&mut self) {
        trace!("parser::Parser::expression()");
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        trace!("parser::Parser::number()");
        let num_str = self.span_to_str(self.previous.literal.clone());
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
        let literal = self.previous.literal.clone();
        self.emit_constant(Value::String {
            value: Rc::new(String::from(
                &self.source[literal.start + 1..literal.end - 1],
            )),
        });
    }

    fn variable(&mut self, can_assign: bool) {
        trace!("parser::Parser::variable()");
        self.named_variable(self.previous.literal.clone(), can_assign);
    }

    fn and(&mut self) {
        trace!("parser::Parser::and()");

        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        trace!("parser::Parser::or()");

        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_op(OpCode::Pop);
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn call(&mut self) {
        trace!("parser::Parser::call()");

        let arg_count = self.argument_list();
        self.emit_ops_usize(OpCode::Call, arg_count);
    }

    fn argument_list(&mut self) -> usize {
        let mut arg_count = 0;

        if !self.check_type(TokenType::RightParen) {
            loop {
                arg_count += 1;
                self.expression();

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments.");

        arg_count
    }

    fn named_variable(&mut self, name: Span, can_assign: bool) {
        trace!("parser::Parser::named_variable(name: {:?}", name);
        let mut arg = self.resolve_local(name.clone());
        let get_op: OpCode;
        let set_op: OpCode;

        if arg != usize::MAX {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else {
            arg = self.identifier_constant(name);
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_ops_usize(set_op, arg);
        } else {
            self.emit_ops_usize(get_op, arg);
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        trace!("parser::Parser::parse_variable()");
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        if self.current_frame().scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.previous.literal.clone())
    }

    fn declare_variable(&mut self) {
        trace!("parser::Parser::declare_variable()");
        if self.current_frame().scope_depth == 0 {
            return;
        }

        let name = self.previous.literal.clone();

        let frame = self.current_frame();

        let has_duplicate = frame
            .locals
            .iter()
            .rev()
            .take_while(|l| l.depth != usize::MAX && l.depth < frame.scope_depth)
            .any(|l| self.identifier_equal(name.clone(), l.name.clone()));

        if has_duplicate {
            self.error_at_current("Already a variable with this name in this scope.");
        }

        self.add_local(name);
    }

    fn add_local(&mut self, name: Span) {
        trace!("parser::Parser::add_local(name: {:?})", name);

        let frame = self.current_frame_mut();
        frame.locals.push(Local::new(name, usize::MAX));
    }

    fn identifier_constant(&mut self, name: Span) -> usize {
        trace!("parser::Parser::identifier_constant(name: {:?})", name);

        let source = &self.source[name.start..name.end];
        let frame = self.current_frame_mut();
        frame.function.chunk.add_constant(Value::String {
            value: Rc::new(String::from(source)),
        })
    }

    fn declaration(&mut self) {
        trace!("parser::Parser::declaration()");
        match self.current.token_type {
            TokenType::Fun => self.fun_declaration(),
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }

        if self.panic {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        trace!("parser::Parser::fun_declaration()");
        self.advance();

        let global = self.parse_variable("Expected function name after fun.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn function(&mut self, function_type: FunctionType) {
        trace!(
            "parser::Parser::function(function_type: {:?})",
            function_type
        );

        let frame = CompileFrame::new(
            Function::new(
                0,
                Chunk::new(),
                Some(self.span_to_str(self.previous.literal.clone()).to_string()),
            ),
            function_type,
            vec![],
            vec![],
            self.current_frame().scope_depth,
        );
        self.frames.push(frame);

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expected '(' after function name.");
        if !self.check_type(TokenType::RightParen) {
            loop {
                let frame = self.current_frame_mut();

                frame.function.arity += 1;

                let constant = self.parse_variable("Expected parameter name.");
                self.define_variable(constant);

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            "Expected ')' after function parameters.",
        );
        self.consume(TokenType::LeftBrace, "Expected '{' before function body.");

        while !self.check_type(TokenType::RightBrace) && !self.check_type(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after function body.");

        #[cfg(feature = "trace_execution")]
        {
            if self.had_error {
                let frame = self.current_frame();
                match &frame.function.name {
                    Some(name) => disassemble_chunk(name.as_str(), &frame.function.chunk),
                    None => disassemble_chunk("<script>", &frame.function.chunk),
                }
            }
        }

        self.emit_op(OpCode::Return);

        let frame = self
            .frames
            .pop()
            .expect("function frame must be present when finishing compilation");
        self.emit_constant(Value::Function {
            value: frame.function,
        });
    }

    fn var_declaration(&mut self) {
        trace!("parser::Parser::var_declaration()");
        self.advance();

        let var = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Nil);
        }

        self.consume(TokenType::Semi, "Expected ';' after variable declaration");

        self.define_variable(var);
    }

    fn define_variable(&mut self, var: usize) {
        trace!("parser::Parser::define_variable(var: {var})");
        if self.current_frame().scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_op(OpCode::DefineGlobal);
        self.emit_op_usize(var);
    }

    fn mark_initialized(&mut self) {
        trace!("parser::Parser::mark_initialized()");
        if self.current_frame().scope_depth == 0 {
            return;
        }
        let frame = self.current_frame_mut();
        let scope_depth = frame.scope_depth;

        frame.locals.last_mut().unwrap().set_depth(scope_depth);
    }

    fn statement(&mut self) {
        trace!("parser::Parser::statement()");
        match self.current.token_type {
            TokenType::Print => self.print_statement(),
            TokenType::If => self.if_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::LeftBrace => self.block(),
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

    fn block(&mut self) {
        trace!("parser::Parser::block()");
        self.begin_scope();

        self.advance();
        while !self.check_type(TokenType::RightBrace) && !self.check_type(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.");

        self.end_scope();
    }

    fn expression_statement(&mut self) {
        trace!("parser::Parser::expression_statement()");
        self.expression();
        self.consume(TokenType::Semi, "Expected ';' after expression.");
        self.emit_op(OpCode::Pop);
    }

    fn begin_scope(&mut self) {
        trace!("parser::Parser::begin_scope()");
        self.current_frame_mut().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        trace!("parser::Parser::end_scope()");
        {
            let frame = self.current_frame_mut();
            frame.scope_depth -= 1;
        }

        loop {
            let should_pop = {
                let f = self.current_frame();
                match f.locals.last() {
                    None => false,
                    Some(local) => local.depth > f.scope_depth,
                }
            };
            if !should_pop {
                break;
            }

            self.emit_op(OpCode::Pop);
            self.current_frame_mut().locals.pop();
        }
    }

    fn identifier_equal(&self, a: Span, b: Span) -> bool {
        trace!("parser::Parser::identifier_equal(a: {:?}, b: {:?})", a, b);

        if a.end - a.start != b.end - b.start {
            false
        } else if self.source[a.start..a.end] == self.source[b.start..b.end] {
            true
        } else {
            false
        }
    }

    fn resolve_local(&mut self, name: Span) -> usize {
        trace!("parser::Parser::resolve_local(name: {:?})", name);
        let mut error = false;
        let frame = self.current_frame();
        let resolved = frame
            .locals
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, local)| {
                if self.identifier_equal(name.clone(), local.name.clone()) {
                    if local.depth == 0 {
                        error = true;
                        None
                    } else {
                        Some(i)
                    }
                } else {
                    None
                }
            })
            .unwrap_or(usize::MAX);

        if error {
            self.error_at_current("Can't read local variable in its own initializer.");
        }
        resolved
    }

    fn if_statement(&mut self) {
        trace!("parser::Parser::if_statement()");
        self.advance();

        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_op(OpCode::Pop);

        if self.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn return_statement(&mut self) {
        trace!("parser::Parser::return_statement()");
        self.advance();

        if self.current_frame().function_type == FunctionType::Script {
            self.error_at_current("Can't return from top-level code.");
            return;
        }

        if self.match_token(TokenType::Semi) {
            self.emit_ops(OpCode::Nil, OpCode::Return);
        } else {
            self.expression();
            self.consume(TokenType::Semi, "Expected ';' after return value.");
            self.emit_op(OpCode::Return);
        }
    }

    fn while_statement(&mut self) {
        trace!("parser::Parser::while_statement()");
        self.advance();

        let frame = self.current_frame();
        let loop_start = frame.function.chunk.len();

        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        trace!("parser::Parser::for_loop()");
        self.advance();

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.");

        if self.match_token(TokenType::Semi) {
            self.advance();
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let frame = self.current_frame();
        let loop_start = frame.function.chunk.len();
        self.consume(
            TokenType::Semi,
            "Expected ';' after loop variable condition.",
        );
        //

        self.consume(TokenType::RightParen, "Expected ')' after condition.");

        self.statement();
        self.emit_loop(loop_start);

        self.end_scope();
    }

    fn emit_loop(&mut self, loop_start: usize) {
        trace!("parser::Parser::emit_loop(loop_start: {loop_start})");

        self.emit_op(OpCode::Loop);

        let frame = self.current_frame();
        let offset = frame.function.chunk.len() - loop_start;

        self.emit_op_usize(offset);
    }
}

impl<'a> Parser<'a> {
    fn emit_op(&mut self, op: OpCode) {
        trace!("parser::Parser::emit_op(op: {:?})", op);
        let line = self.previous.line;
        let col = self.previous.col;
        let function = &mut self.current_frame_mut().function;
        function.chunk.write(op as usize, line, col);
    }

    fn emit_op_usize(&mut self, op: usize) {
        trace!("parser::Parser::emit_op(op: {op})");
        let line = self.previous.line;
        let col = self.previous.col;
        let function = &mut self.current_frame_mut().function;
        function.chunk.write(op, line, col);
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
        let line = self.previous.line;
        let col = self.previous.col;
        let function = &mut self.current_frame_mut().function;
        let pos = function.chunk.add_constant(value);
        function.chunk.write(OpCode::Constant as usize, line, col);
        function.chunk.write(pos, line, col);
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        trace!("parser::Parser::emit_jump({:?})", op);
        self.emit_op(op);
        self.emit_op_usize(usize::MAX);
        let function = &mut self.current_frame_mut().function;
        function.chunk.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        trace!("parser::Parser::patch_jump(offset: {offset})");
        // Distance from the operand to the next instruction after the jump target
        let function = &mut self.current_frame_mut().function;
        let jump = function.chunk.len() - offset - 1;
        function.chunk.instructions[offset] = jump;
    }
}
