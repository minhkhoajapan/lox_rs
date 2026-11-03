use std::{env, fs, process};

use crate::{
    error::error::{had_error, had_runtime_error},
    parser::parser::Parser,
    runtime::interpreter::Interpreter,
    utils::scanner::scan_tokens,
};

mod utils;
mod error;
mod parser;
mod runtime;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: lox_rs <script.lox>");
        process::exit(64);
    });

    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Can not read {}: {}", path, e);
        process::exit(66);
    });

    let tokens = scan_tokens(&source);
    let expr = Parser::new(&tokens).parse();

    if had_error() {
        process::exit(65);
    }

    if let Some(expr) = expr {
        Interpreter::new().interpret(&expr);
    }

    if had_runtime_error() {
        process::exit(70);
    }
}
