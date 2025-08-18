mod debug;
mod value;
mod vm;

use std::env;
use std::process;

use crate::debug::disassemble_chunk;

fn print_error(message: &str) {
    println!("{message}");
}

fn main() {
    let mut args = env::args();

    let (Some(_), Some(source)) = (args.next(), args.next()) else {
        print_error("Usage: lox Source");
        process::exit(0);
    };

    println!("{source}");

    let mut vm = vm::VM::new();

    let constant = vm.chunk.add_constant(1.0);
    vm.chunk
        .write(vm::OpCode::Constant as usize, vm::Loc::new(0, 0));
    vm.chunk.write(constant, vm::Loc::new(0, 0));
    vm.chunk
        .write(vm::OpCode::Return as usize, vm::Loc::new(0, 1));
    vm.chunk
        .write(vm::OpCode::Return as usize, vm::Loc::new(1, 0));
    disassemble_chunk("Debug", &vm.chunk);
    vm.free();
}
