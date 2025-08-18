use crate::value;
use crate::vm::{Chunk, OpCode};

pub fn disassemble_chunk(name: &str, chunk: &Chunk) {
    println!("===   {name}   ===");

    let mut offset: usize = 0;
    loop {
        if offset >= chunk.instructions.len() {
            break;
        }
        offset = disassemble_instruction(&chunk, offset);
    }

    println!("=== {name} end ===");
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    let instruction = OpCode::from_usize(chunk.instructions[offset]);
    let loc = &chunk.loc[offset];

    print!("{offset:0>4} [{}:{}] ", loc.line, loc.col);

    match instruction {
        OpCode::Return => simple_instruction("OP_RETURN", offset),
        OpCode::Constant => constant_instruction(&chunk, offset),
        OpCode::Negate => simple_instruction("OP_NEGATE", offset),
        OpCode::Add => simple_instruction("OP_ADD", offset),
        OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
        OpCode::Unknown => {
            println!("Unknown opcode {:?}", instruction);
            offset + 1
        }
    }
}

pub fn simple_instruction(instruction: &str, offset: usize) -> usize {
    println!("{instruction}");
    offset + 1
}

pub fn constant_instruction(chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.instructions[offset + 1];
    let value = chunk.constants[constant];

    print!("OP_CONSTANT {constant:0>4} ");
    value::print_value(value);
    println!();
    offset + 2
}
