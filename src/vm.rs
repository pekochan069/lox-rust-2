use log::trace;
use std::collections::HashMap;
use std::rc::Rc;

use lox_rust_2::{binary_bool_op, binary_number_op};

use crate::args::Args;
use crate::compile::compile;
use crate::token::Span;
use crate::value::Value;

macro_rules! try_or_return {
    ($expr:expr) => {
        if let Err(e) = $expr {
            return e;
        }
    };
}

#[derive(Debug)]
#[repr(usize)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    Unknown,
}

impl OpCode {
    pub fn from_usize(num: usize) -> Self {
        trace!("vm::OpCode::from_usize(num: {num})");
        match num {
            0 => Self::Return,
            1 => Self::Constant,
            2 => Self::Negate,
            3 => Self::Not,
            4 => Self::Add,
            5 => Self::Subtract,
            6 => Self::Multiply,
            7 => Self::Divide,
            8 => Self::Nil,
            9 => Self::True,
            10 => Self::False,
            11 => Self::Equal,
            12 => Self::Greater,
            13 => Self::Less,
            14 => Self::Print,
            15 => Self::Pop,
            16 => Self::DefineGlobal,
            17 => Self::GetGlobal,
            18 => Self::SetGlobal,
            19 => Self::GetLocal,
            20 => Self::SetLocal,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize) -> Self {
        trace!("vm::Loc::new(line: {line}, col: {col})");
        Self { line, col }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub instructions: Vec<usize>,
    pub constants: Vec<Value>,
    pub loc: Vec<Loc>,
}

impl Chunk {
    pub fn new() -> Self {
        trace!("vm::Chunk::new()");
        Self {
            instructions: vec![],
            constants: vec![],
            loc: vec![],
        }
    }

    pub fn write(&mut self, op: usize, line: usize, col: usize) {
        trace!("vm::Chunk::write(op: {op}, line: {line}, col: {col})");
        self.instructions.push(op);
        self.loc.push(Loc::new(line, col));
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        trace!("vm::Chunk::add_constant(value: {value})");
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn clear(&mut self) {
        trace!("vm::Chunk::clear()");
        self.instructions.clear();
        self.constants.clear();
        self.loc.clear();
    }
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    pub chunk: Chunk,
    cursor: usize,
    stack: Vec<Value>,
    source: String,
    globals: HashMap<Rc<String>, Value>,
}

impl VM {
    pub fn new(args: &Args) -> Self {
        trace!("vm::VM::new(args: {:?})", args);
        Self {
            chunk: Chunk::new(),
            cursor: 0,
            stack: vec![],
            source: String::new(),
            globals: HashMap::new(),
        }
    }

    pub fn free(&mut self) {
        trace!("vm::VM::free()");
        self.chunk.clear();
        self.stack.clear();
        self.globals.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        trace!("vm::VM::interpret()");
        self.source = source;
        self.cursor = 0;

        let Ok(_ew_chunk) = compile(self.source.as_str(), &mut self.chunk) else {
            return InterpretResult::CompileError;
        };

        self.run()
    }
}

impl VM {
    fn run(&mut self) -> InterpretResult {
        trace!("vm::VM::run()");

        loop {
            #[cfg(feature = "trace_execution")]
            {
                // disassemble
                crate::debug::disassemble_instruction(&self.chunk, self.cursor);

                // stack trace
                if self.stack.len() > 0 {
                    print!("  Stack - ");
                    for value in self.stack.iter() {
                        print!("[ {} ]", *value);
                    }
                    println!();
                }
            }

            let Some(instruction) = self.next_opcode() else {
                break;
            };

            match instruction {
                OpCode::Return => return InterpretResult::Ok,
                OpCode::Constant => try_or_return!(self.constant()),
                OpCode::Negate => try_or_return!(self.negate()),
                OpCode::Not => try_or_return!(self.not()),
                OpCode::Add => try_or_return!(self.add()),
                OpCode::Subtract => try_or_return!(self.subtract()),
                OpCode::Multiply => try_or_return!(self.multiply()),
                OpCode::Divide => try_or_return!(self.divide()),
                OpCode::Nil => self.push_value(Value::Nil),
                OpCode::True => self.push_value(Value::Bool { value: true }),
                OpCode::False => self.push_value(Value::Bool { value: false }),
                OpCode::Equal => try_or_return!(self.equal()),
                OpCode::Greater => try_or_return!(self.greater()),
                OpCode::Less => try_or_return!(self.less()),
                OpCode::Print => try_or_return!(self.print()),
                OpCode::Pop => {
                    if let None = self.pop_value() {
                        return self.runtime_error("Invalid access to stack.");
                    }
                }
                OpCode::DefineGlobal => try_or_return!(self.define_global()),
                OpCode::GetGlobal => try_or_return!(self.get_global()),
                OpCode::SetGlobal => try_or_return!(self.set_global()),
                OpCode::GetLocal => try_or_return!(self.get_local()),
                OpCode::SetLocal => try_or_return!(self.set_local()),
                OpCode::Unknown => return InterpretResult::CompileError,
            }

            if self.cursor == self.chunk.instructions.len() {
                break;
            }
        }

        InterpretResult::Ok
    }

    fn next(&mut self) -> Option<usize> {
        trace!("vm::VM::next()");
        if self.chunk.instructions.len() == 0 {
            None
        } else {
            let instruction = self.chunk.instructions[self.cursor];
            self.cursor += 1;
            Some(instruction)
        }
    }

    fn next_opcode(&mut self) -> Option<OpCode> {
        trace!("VM::next_opcode()");
        if self.chunk.instructions.len() == 0 {
            None
        } else {
            let instruction = OpCode::from_usize(self.chunk.instructions[self.cursor]);
            self.cursor += 1;
            Some(instruction)
        }
    }

    fn push_value(&mut self, value: Value) {
        trace!("vm::VM::push_value(value: {value})");
        self.stack.push(value);
    }

    fn pop_value(&mut self) -> Option<Value> {
        trace!("vm::VM::pop_value()");
        self.stack.pop()
    }

    fn top_value(&mut self) -> Option<Value> {
        trace!("vm::VM::top_value()");
        if self.stack.len() == 0 {
            None
        } else {
            Some(self.stack[self.stack.len() - 1].clone())
        }
    }

    fn peek_value_at(&self, at: usize) -> Option<&Value> {
        if at < self.stack.len() {
            Some(&self.stack[self.stack.len() - at - 1])
        } else {
            None
        }
    }

    fn read_constant(&mut self) -> Result<Value, ()> {
        trace!("vm::VM::read_constant()");
        let Some(constant_location) = self.next() else {
            return Err(());
        };

        let value = self.chunk.constants[constant_location].clone();

        Ok(value)
    }

    fn constant(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::constant()");
        let Ok(constant) = self.read_constant() else {
            return Err(InterpretResult::RuntimeError);
        };
        self.push_value(constant);
        Ok(())
    }

    fn negate(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::negate()");
        let Some(popped_value) = self.pop_value() else {
            return Err(self.runtime_error("Invalid access to stack"));
        };

        match popped_value {
            Value::Number { value } => self.push_value(Value::Number { value: -value }),
            _ => {
                return Err(self.runtime_error("Operand must be a number."));
            }
        }

        Ok(())
    }

    fn not(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::not()");
        let Some(popped_value) = self.pop_value() else {
            return Err(self.runtime_error("Invalid access to stack"));
        };

        self.push_value(Value::Bool {
            value: popped_value.is_falsy(),
        });

        Ok(())
    }

    fn add(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::add()");
        let (Some(b), Some(a)) = (self.peek_value_at(0), self.peek_value_at(1)) else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        match (a, b) {
            (Value::Number { value: _ }, Value::Number { value: _ }) => {
                binary_number_op!(+);
            }
            (Value::String { value: _ }, Value::String { value: _ }) => {
                let b = self.pop_value().unwrap();
                let a = self.pop_value().unwrap();

                match (a, b) {
                    (Value::String { value: a_value }, Value::String { value: b_value }) => {
                        let concatenated = format!("{}{}", a_value, b_value);
                        self.push_value(Value::String {
                            value: Rc::new(concatenated),
                        });
                    }
                    _ => {}
                }
            }
            _ => {
                return Err(self.runtime_error("Operands must be two numbers or two strings."));
            }
        }

        Ok(())
    }

    fn subtract(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::subtract()");
        binary_number_op!(-);
        Ok(())
    }

    fn multiply(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::multiply()");
        binary_number_op!(*);
        Ok(())
    }

    fn divide(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::divide()");
        binary_number_op!(/);
        Ok(())
    }

    fn equal(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::equal()");
        let (Some(b), Some(a)) = (self.pop_value(), self.pop_value()) else {
            return Err(self.runtime_error("Invalid access to stack"));
        };

        self.push_value(Value::Bool { value: a.eq(b) });

        Ok(())
    }

    fn greater(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::greater()");
        binary_bool_op!(>);
        Ok(())
    }

    fn less(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::less()");
        binary_bool_op!(<);
        Ok(())
    }

    fn print(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::print()");
        let Some(popped_value) = self.pop_value() else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        println!("{popped_value}");
        Ok(())
    }

    fn define_global(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::define_global()");
        let Ok(name) = self.read_constant() else {
            return Err(self.runtime_error("Identifier name not found."));
        };

        match name {
            Value::String { value } => {
                let Some(peeked) = self.peek_value_at(0) else {
                    return Err(self.runtime_error("Invalid access to stack."));
                };

                self.globals.insert(value.clone(), peeked.clone());
                _ = self.pop_value().unwrap();
            }
            _ => return Err(self.runtime_error("Invalid name for identifier.")),
        }

        Ok(())
    }

    fn get_global(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::get_global()");
        let Ok(name_value) = self.read_constant() else {
            return Err(self.runtime_error("Identifier name not found."));
        };

        match name_value {
            Value::String { value: name } => {
                let Some(value) = self.globals.get(&name) else {
                    return Err(
                        self.runtime_error(format!("Global Variable {name} not found").as_str())
                    );
                };
                self.push_value(value.clone());
            }
            _ => return Err(self.runtime_error("Invalid name for identifier.")),
        }

        Ok(())
    }

    fn set_global(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::set_global()");
        let Ok(name_value) = self.read_constant() else {
            return Err(self.runtime_error("Identifier name not found."));
        };

        let Value::String { value: name } = name_value else {
            return Err(self.runtime_error("Invalid name for identifier."));
        };

        let Some(value) = self.peek_value_at(0) else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        if !self.globals.contains_key(&name) {
            return Err(self.runtime_error(&format!("Undefined variable {}.", name)));
        }

        *self.globals.get_mut(&name).unwrap() = value.clone();

        Ok(())
    }

    fn get_local(&mut self) -> Result<(), InterpretResult> {
        let Some(slot) = self.next() else {
            return Err(self.runtime_error("Cannot get slot for local variable."));
        };

        self.push_value(self.peek_value_at(slot).unwrap().clone());

        Ok(())
    }

    fn set_local(&mut self) -> Result<(), InterpretResult> {
        let Some(slot) = self.next() else {
            return Err(self.runtime_error("Cannot get slot for local variable."));
        };

        self.stack[slot] = self.peek_value_at(0).unwrap().clone();

        Ok(())
    }
}

impl VM {
    fn runtime_error(&self, message: &str) -> InterpretResult {
        let loc = self.chunk.loc[self.cursor].clone();
        eprintln!("[{}:{}] {message}", loc.line, loc.col);
        InterpretResult::RuntimeError
    }
}
