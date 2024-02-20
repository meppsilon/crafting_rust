mod environment;
mod expression;
mod interpreter;
mod parser;
mod scanner;
mod statement;
mod token;
mod function;
mod returns;
mod resolver;
mod lox_class;

use interpreter::Interpreter;

use crate::parser::*;
use crate::scanner::Scanner;
use crate::token::*;
use std::io::prelude::*;
use std::{env, fs, io, process};

static mut HAD_ERROR: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();
    let length: usize = args.len();

    if length > 2 {
        println!("Usage: jlox [script]");
    } else if length == 2 {
        println!("arg: {}", &args[1]);
        run_file(&args[1]);
    } else {
        println!("no args");
        run_prompt();
    }
}

fn run_file(path: &str) {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");
    run(&contents);
    unsafe {
        if HAD_ERROR {
            process::exit(0);
        }
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().len() == 0 {
            break;
        }
        run(input.trim());
        unsafe {
            HAD_ERROR = false;
        }
    }
}

fn run(source: &str) {
    let mut interpreter = Interpreter::new();

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    interpreter.interpret(statements.clone());

    unsafe {
        if HAD_ERROR {
            return;
        }
    }
}

fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn error_at_token(token: &Token, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

fn report(line: u32, where1: &str, message: &str) {
    println!("[line {line}] Error{where1}: {message}");
    unsafe {
        HAD_ERROR = true;
    }
}
