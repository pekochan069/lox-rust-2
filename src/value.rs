use std::{fmt, rc::Rc};

use crate::function::Function;

#[derive(Debug, Clone)]
pub struct NativeFn {
    pub name: String,
    pub function: fn(usize, Vec<Value>) -> Value,
}

impl NativeFn {
    pub fn new(name: &str, function: fn(usize, Vec<Value>) -> Value) -> Self {
        Self {
            name: String::from(name),
            function,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool { value: bool },
    Number { value: f64 },
    Nil,
    String { value: Rc<String> },
    Function { value: Function },
    NativeFn { value: NativeFn },
}

impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Self::Bool { value } => !value,
            Self::Number { value: _ } => false,
            Self::Nil => true,
            Self::String { value: _ } => false,
            Self::Function { value: _ } => false,
            Self::NativeFn { value: _ } => false,
        }
    }

    pub fn eq(&self, other: Value) -> bool {
        match (self, other) {
            (Self::Bool { value: a }, Self::Bool { value: b }) => *a == b,
            (Self::Number { value: a }, Self::Number { value: b }) => *a == b,
            (Self::Nil, Self::Nil) => true,
            (Self::String { value: a }, Self::String { value: b }) => *a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool { value } => write!(f, "{value}"),
            Self::Number { value } => write!(f, "{value}"),
            Self::Nil => write!(f, "nil"),
            Self::String { value } => write!(f, "{value}"),
            Self::Function { value } => {
                let name = &value.name;

                match name {
                    Some(name) => write!(f, "<fn {name}>"),
                    None => write!(f, "<script>"),
                }
            }
            Self::NativeFn { value } => {
                write!(f, "<native fn {}>", value.name)
            }
        }
    }
}
