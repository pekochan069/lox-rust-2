use log::trace;
use lox_rust_2::binary_op;

use crate::args::Args;
use crate::value::{Value, print_value};

#[derive(Debug)]
#[repr(usize)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Unknown,
}

impl OpCode {
    pub fn from_usize(num: usize) -> Self {
        trace!("OpCode::from_usize(num: {num})");
        match num {
            0 => Self::Return,
            1 => Self::Constant,
            2 => Self::Negate,
            3 => Self::Add,
            4 => Self::Subtract,
            5 => Self::Multiply,
            6 => Self::Divide,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

impl Loc {
    pub fn new(line: usize, col: usize) -> Self {
        trace!("Loc::new(line: {line}, col: {col})");
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
        trace!("Chunk::new()");
        Self {
            instructions: vec![],
            constants: vec![],
            loc: vec![],
        }
    }

    pub fn write(&mut self, op: usize, loc: Loc) {
        trace!("Chunk::write(op: {op}, loc: {:?})", loc);
        self.instructions.push(op);
        self.loc.push(loc);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        trace!("Chunk::add_constant(value: {value})");
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn clear(&mut self) {
        trace!("Chunk::clear()");
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
}

impl VM {
    pub fn new(args: &Args) -> Self {
        trace!("VM::new(args: {:?}", args);
        Self {
            chunk: Chunk::new(),
            cursor: 0,
            stack: vec![],
            source: String::new(),
        }
    }

    pub fn free(&mut self) {
        trace!("VM::free()");
        self.chunk.clear();
        self.stack.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        trace!("VM::interpret()");
        self.source = source;
        self.cursor = 0;
        self.run()
    }
}

impl VM {
    fn set_chunk(&mut self, chunk: Chunk) {
        trace!("VM::set_chunk(chunk: {:?})", chunk);
        self.chunk = chunk;
    }

    fn run(&mut self) -> InterpretResult {
        trace!("VM::run()");
        loop {
            #[cfg(feature = "trace_execution")]
            {
                // disassemble
                crate::debug::disassemble_instruction(&self.chunk, self.cursor);

                // stack trace
                if self.stack.len() > 0 {
                    print!("  Stack - ");
                    for value in self.stack.iter() {
                        print!("[ ");
                        print_value(*value);
                        print!(" ]");
                    }
                    println!();
                }
            }

            let instruction = self.next_opcode();

            match instruction {
                OpCode::Return => {
                    let Some(popped_value) = self.pop_value() else {
                        return InterpretResult::RuntimeError;
                    };
                    print_value(popped_value);
                    println!();
                    return InterpretResult::Ok;
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push_value(constant);
                }
                OpCode::Negate => {
                    let Some(popped_value) = self.pop_value() else {
                        return InterpretResult::RuntimeError;
                    };
                    self.push_value(-popped_value);
                }
                OpCode::Add => binary_op!(+),
                OpCode::Subtract => binary_op!(-),
                OpCode::Multiply => binary_op!(*),
                OpCode::Divide => binary_op!(/),
                OpCode::Unknown => return InterpretResult::CompileError,
            }
        }
    }

    fn next(&mut self) -> usize {
        trace!("VM::next()");
        let instruction = self.chunk.instructions[self.cursor];
        self.cursor += 1;
        instruction
    }

    fn next_opcode(&mut self) -> OpCode {
        trace!("VM::next_opcode()");
        let instruction = OpCode::from_usize(self.chunk.instructions[self.cursor]);
        self.cursor += 1;
        instruction
    }

    fn push_value(&mut self, value: Value) {
        trace!("VM::push_value(value: {value})");
        self.stack.push(value);
    }

    fn pop_value(&mut self) -> Option<Value> {
        trace!("VM::pop_value()");
        self.stack.pop()
    }

    fn top_value(&mut self) -> Option<Value> {
        trace!("VM::top_value()");
        if self.stack.len() == 0 {
            None
        } else {
            Some(self.stack[self.stack.len() - 1])
        }
    }

    fn read_constant(&mut self) -> Value {
        trace!("VM::read_constant()");
        let constant_location = self.next();
        let value = self.chunk.constants[constant_location];

        value
    }
}
