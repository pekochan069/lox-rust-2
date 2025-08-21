#![feature(gen_blocks)]
mod args;
mod compile;
mod debug;
mod error;
mod lexer;
mod logger;
mod parser;
mod token;
mod value;
mod vm;

use std::{
    fs,
    io::{self, Write},
    process,
};

use clap::Parser;

use args::Args;
use logger::init_logger;
use vm::{InterpretResult, VM};

fn main() {
    let args = args::Args::parse();

    let Ok(_) = init_logger(args.log_level) else {
        println!("Failed to initialize logger");
        process::exit(64);
    };

    match &args.source {
        Some(s) => run_file(&args, s.as_str()),
        None => repl(&args),
    }
}
fn run_file(args: &Args, path: &str) {
    let Ok(source) = fs::read_to_string(&path) else {
        log::error!("Cannot read file from {path}");
        process::exit(74);
    };

    // let mut lexer = Lexer::new(source.as_str());

    // for maybe_token in lexer.iter() {
    //     match maybe_token {
    //         Ok(token) => println!("{:?}", token),
    //         Err(e) => e.report(),
    //     }
    // }

    let mut vm = VM::new(args);

    let result = vm.interpret(source);

    vm.free();

    match result {
        InterpretResult::CompileError => process::exit(65),
        InterpretResult::RuntimeError => process::exit(70),
        InterpretResult::Ok => return,
    }
}

fn repl(args: &Args) {
    let mut vm = VM::new(&args);
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input_string = String::new();
        io::stdin()
            .read_line(&mut input_string)
            .expect("Failed to read input");

        if input_string.eq("exit") {
            break;
        }

        vm.interpret(input_string);
    }
    vm.free();
}
