use log::trace;
use std::collections::HashMap;
use std::rc::Rc;

use lox_rust_2::{binary_bool_op, binary_number_op};

use crate::args::Args;
use crate::compile::compile;
use crate::function::Function;
use crate::value::{NativeFn, Value};

static MAX_FRAMES: usize = 255;

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
    JumpIfFalse,
    Jump,
    Loop,
    Call,
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
            21 => Self::JumpIfFalse,
            22 => Self::Jump,
            23 => Self::Loop,
            24 => Self::Call,
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

#[derive(Debug, Clone)]
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

    pub fn len(&self) -> usize {
        trace!("vm::Chunk::len()");
        self.instructions.len()
    }
}

#[derive(Debug)]
pub struct CallFrame {
    function: Function,
    cursor: usize,
    slot_base: usize,
}

impl CallFrame {
    pub fn new(function: Function, cursor: usize, slot_base: usize) -> Self {
        Self {
            function,
            cursor,
            slot_base,
        }
    }
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    stack: Vec<Value>,
    source: String,
    globals: HashMap<Rc<String>, Value>,
    frames: Vec<CallFrame>,
}

impl VM {
    pub fn new(args: &Args) -> Self {
        trace!("vm::VM::new(args: {:?})", args);

        Self {
            stack: vec![],
            source: String::new(),
            globals: HashMap::new(),
            frames: vec![],
        }
    }

    pub fn free(&mut self) {
        trace!("vm::VM::free()");
        self.frames.clear();
        self.stack.clear();
        self.globals.clear();
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        trace!("vm::VM::interpret()");

        self.populate_native_fn();

        self.source = source;

        let Ok(function) = compile(self.source.as_str()) else {
            return InterpretResult::CompileError;
        };

        self.frames.push(CallFrame::new(function, 0, 0));

        self.run()
    }
}

impl VM {
    fn populate_native_fn(&mut self) {
        self.define_native("hello", |_, _| Value::String {
            value: Rc::new(String::from("Hello, world!")),
        });
    }

    fn run(&mut self) -> InterpretResult {
        trace!("vm::VM::run()");

        loop {
            #[cfg(feature = "trace_execution")]
            {
                // disassemble
                crate::debug::disassemble_instruction(self.current_chunk(), self.current_cursor());

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
                OpCode::Return => {
                    let result = self.return_op();

                    match result {
                        Ok(interpret_finished) => {
                            if interpret_finished {
                                return InterpretResult::Ok;
                            }
                        }
                        Err(e) => return e,
                    }
                }
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
                OpCode::JumpIfFalse => try_or_return!(self.jump_if_false()),
                OpCode::Jump => self.jump(),
                OpCode::Loop => self.loop_op(),
                OpCode::Call => try_or_return!(self.call_op()),
                OpCode::Unknown => return InterpretResult::CompileError,
            }

            if self.current_cursor() == self.current_instructions().len() {
                break;
            }
        }

        InterpretResult::Ok
    }

    fn next(&mut self) -> Option<usize> {
        trace!("vm::VM::next()");
        let frame = self.current_frame_mut();
        if frame.function.chunk.instructions.len() == 0 {
            None
        } else {
            let instruction = frame.function.chunk.instructions[frame.cursor];
            frame.cursor += 1;
            Some(instruction)
        }
    }

    fn next_opcode(&mut self) -> Option<OpCode> {
        trace!("VM::next_opcode()");
        let frame = self.current_frame_mut();
        if frame.function.chunk.instructions.len() == 0 {
            None
        } else {
            let instruction = OpCode::from_usize(frame.function.chunk.instructions[frame.cursor]);
            frame.cursor += 1;
            Some(instruction)
        }
    }

    #[inline]
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    #[inline]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    #[inline]
    fn current_cursor(&self) -> usize {
        self.frames.last().unwrap().cursor
    }

    #[inline]
    fn current_slot_base(&self) -> usize {
        self.frames
            .last()
            .expect("call frame must exist when accessing slot base")
            .slot_base
    }

    #[inline]
    fn current_function(&self) -> &Function {
        let frame = self.frames.last().unwrap();
        &frame.function
    }

    #[inline]
    fn current_function_mut(&mut self) -> &mut Function {
        let frame = self.frames.last_mut().unwrap();
        &mut frame.function
    }

    #[inline]
    fn current_chunk(&self) -> &Chunk {
        let function = &(self.frames.last().unwrap().function);
        &function.chunk
    }

    #[inline]
    fn current_chunk_mut(&mut self) -> &mut Chunk {
        let function = &mut (self.frames.last_mut().unwrap().function);
        &mut function.chunk
    }

    #[inline]
    fn current_instructions(&self) -> &Vec<usize> {
        let chunk = &(self.frames.last().unwrap().function.chunk);
        &chunk.instructions
    }

    #[inline]
    fn current_instructions_mut(&mut self) -> &mut Vec<usize> {
        let chunk = &mut (self.frames.last_mut().unwrap().function.chunk);
        &mut chunk.instructions
    }

    #[inline]
    fn current_instruction(&self) -> usize {
        self.current_instructions()[self.current_cursor()]
    }

    fn push_value(&mut self, value: Value) {
        trace!("vm::VM::push_value(value: {value})");
        self.stack.push(value);
    }

    fn pop_value(&mut self) -> Option<Value> {
        trace!("vm::VM::pop_value()");
        self.stack.pop()
    }

    fn top_value(&mut self) -> Option<&Value> {
        trace!("vm::VM::top_value()");
        if self.stack.len() == 0 {
            None
        } else {
            Some(&self.stack[self.stack.len() - 1])
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

        let value = self.current_chunk().constants[constant_location].clone();

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
        trace!("vm::VM::get_local()");
        let Some(slot) = self.next() else {
            return Err(self.runtime_error("Cannot get slot for local variable."));
        };

        let base = self.current_slot_base();
        let index = base + slot;

        let Some(value) = self.stack.get(index) else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        self.push_value(value.clone());

        Ok(())
    }

    fn set_local(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::set_local()");
        let Some(slot) = self.next() else {
            return Err(self.runtime_error("Cannot get slot for local variable."));
        };

        let value = self
            .peek_value_at(0)
            .ok_or_else(|| self.runtime_error("Invalid access to stack."))?
            .clone();

        let base = self.current_slot_base();
        let index = base + slot;

        let Some(slot_ref) = self.stack.get_mut(index) else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        *slot_ref = value;

        Ok(())
    }

    fn jump_if_false(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::jump_if_false()");

        let offset = self.current_instruction();
        {
            self.current_frame_mut().cursor += 1;
        }

        let Some(predicate) = self.peek_value_at(0) else {
            return Err(self.runtime_error("Invalid predicate."));
        };

        if predicate.is_falsy() {
            self.current_frame_mut().cursor += offset;
        }

        Ok(())
    }

    fn jump(&mut self) {
        trace!("vm::VM::jump()");

        let offset = self.current_instruction();
        self.current_frame_mut().cursor += offset + 1;
    }

    fn loop_op(&mut self) {
        trace!("vm::VM::loop_op()");

        let offset = self.current_instruction();
        self.current_frame_mut().cursor -= offset;
    }

    fn call_op(&mut self) -> Result<(), InterpretResult> {
        trace!("vm::VM::call()");

        let arg_count = self.current_instruction();
        self.current_frame_mut().cursor += 1;

        let value = self
            .peek_value_at(arg_count)
            .expect("Invalid access to stack.")
            .clone();

        if !self.call_value(value, arg_count) {
            return Err(InterpretResult::RuntimeError);
        }

        Ok(())
    }

    fn call_value(&mut self, value: Value, arg_count: usize) -> bool {
        trace!("vm::VM::call_value(value: {value}, arg_count: {arg_count})");
        match value {
            Value::Function { value } => self.call(value, arg_count),
            Value::NativeFn { value } => {
                if self.stack.len() < arg_count + 1 {
                    _ = self.runtime_error("Invalid access to stack.");
                    return false;
                }

                let args_start = self.stack.len() - arg_count;
                let args = self.stack[args_start..].to_vec();
                let result = (value.function)(arg_count, args);

                for _ in 0..=arg_count {
                    self.stack.pop();
                }

                self.push_value(result);

                true
            }
            _ => {
                _ = self.runtime_error("Can only call functions and classes.");
                false
            }
        }
    }

    fn call(&mut self, function: Function, arg_count: usize) -> bool {
        trace!("vm::VM::call(function, arg_count: {arg_count})");
        if function.arity != arg_count {
            let message = format!(
                "Expected {} arguments but got {}.",
                function.arity, arg_count
            );
            _ = self.runtime_error(message.as_str());
            return false;
        }

        if self.frames.len() == MAX_FRAMES {
            _ = self.runtime_error("Stack overflow.");
            return false;
        }

        if self.stack.len() < arg_count + 1 {
            _ = self.runtime_error("Invalid access to stack.");
            return false;
        }

        let slot_base = self.stack.len() - arg_count - 1;
        let frame = CallFrame::new(function, 0, slot_base);
        self.frames.push(frame);

        true
    }

    fn return_op(&mut self) -> Result<bool, InterpretResult> {
        trace!("vm::VM::return_op()");
        let Some(result) = self.pop_value() else {
            return Err(self.runtime_error("Invalid access to stack."));
        };

        let frame = self
            .frames
            .pop()
            .expect("return_op cannot run without an active call frame");

        if self.frames.len() == 0 {
            self.stack.clear();
            return Ok(true);
        }

        self.stack.truncate(frame.slot_base);
        self.push_value(result);

        Ok(false)
    }

    fn define_native(&mut self, name: &str, function: fn(usize, Vec<Value>) -> Value) {
        let function = NativeFn::new(name, function);

        self.push_value(Value::String {
            value: Rc::new(String::from(name)),
        });
        self.push_value(Value::NativeFn { value: function });
        let function = self.peek_value_at(0).unwrap();
        let name = self.peek_value_at(1).unwrap();

        match name {
            Value::String { value: name } => {
                self.globals.insert(name.clone(), function.clone());
            }
            _ => {}
        }

        self.pop_value();
        self.pop_value();
    }
}

impl VM {
    fn runtime_error(&self, message: &str) -> InterpretResult {
        let loc = self.current_chunk().loc[self.current_cursor()].clone();
        eprintln!("[{}:{}] {message}", loc.line, loc.col);

        for frame in self.frames.iter() {
            let function = &frame.function;
            let instruction = frame.cursor - 1;
            let loc = function.chunk.loc[instruction].clone();
            eprint!("[line {}] in ", loc.line);

            match &function.name {
                Some(name) => eprintln!("{}", name),
                None => eprintln!("script"),
            }
        }

        InterpretResult::RuntimeError
    }
}
