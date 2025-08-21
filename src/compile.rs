use log::trace;

use crate::error::LoxError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::Chunk;

pub fn compile<'a>(source: &'a str, chunk: &mut Chunk) -> Result<(), LoxError> {
    trace!("compile::compile(source, chunk: {:?}", chunk);
    let mut lexer = Lexer::new(source);
    let mut parser = Parser::new(source, lexer.iter().peekable(), chunk);
    _ = parser.parse()?;

    Ok(())
}
