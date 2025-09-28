use crate::vm::Chunk;

#[derive(Debug, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl Function {
    pub fn new(arity: usize, chunk: Chunk, name: Option<String>) -> Self {
        Self { arity, chunk, name }
    }
}
