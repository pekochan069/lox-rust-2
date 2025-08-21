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
        OpCode::Constant => constant_instruction("OP_CONSTANT", &chunk, offset),
        OpCode::Negate => simple_instruction("OP_NEGATE", offset),
        OpCode::Not => simple_instruction("OP_NOT", offset),
        OpCode::Add => simple_instruction("OP_ADD", offset),
        OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
        OpCode::Nil => simple_instruction("OP_NIL", offset),
        OpCode::True => simple_instruction("OP_TRUE", offset),
        OpCode::False => simple_instruction("OP_FALSE", offset),
        OpCode::Equal => simple_instruction("OP_EQUAL", offset),
        OpCode::Greater => simple_instruction("OP_GREATER", offset),
        OpCode::Less => simple_instruction("OP_LESS", offset),
        OpCode::Print => simple_instruction("OP_PRINT", offset),
        OpCode::Pop => simple_instruction("OP_POP", offset),
        OpCode::DefineGlobal => constant_instruction("OP_DEFINE_GLOBAL", &chunk, offset),
        OpCode::GetGlobal => constant_instruction("OP_GET_GLOBAL", &chunk, offset),
        OpCode::SetGlobal => constant_instruction("OP_SET_GLOBAL", &chunk, offset),
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

pub fn constant_instruction(instruction: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.instructions[offset + 1];
    let value = &chunk.constants[constant];

    print!("{instruction} {constant:0>4} {}", value);
    println!();
    offset + 2
}
