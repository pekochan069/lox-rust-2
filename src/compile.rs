use log::trace;

use crate::error::LoxError;
use crate::function::Function;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn compile<'a>(source: &'a str) -> Result<Function, LoxError> {
    trace!("compile::compile(source)");
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(source, lexer.iter().peekable());
    let frame = parser.parse()?;

    Ok(frame)
}
