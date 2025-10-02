use std::rc::Rc;

use crate::{value::Value, vm::Chunk};

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
    pub upvalue_count: usize,
}

impl Function {
    pub fn new(arity: usize, chunk: Chunk, name: Option<String>, upvalue_count: usize) -> Self {
        Self {
            arity,
            chunk,
            name,
            upvalue_count,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpvalueLoc {
    Open(usize),
    Closed,
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub loc: UpvalueLoc,
    pub value: Rc<Value>,
}

impl Upvalue {
    pub fn new_open(index: usize) -> Self {
        Self {
            loc: UpvalueLoc::Open(index),
            value: Rc::new(Value::Nil),
        }
    }

    pub fn new_closed(value: Rc<Value>) -> Self {
        Self {
            loc: UpvalueLoc::Closed,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<usize>,
}

impl Closure {
    pub fn new(function: Function, upvalues: Vec<usize>) -> Self {
        Self { function, upvalues }
    }
}
