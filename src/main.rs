use std::{env, fs, process};

use crate::{error::error::had_error, utils::scanner::scan_tokens};

mod utils;
mod error;
mod ast;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: lox_rs <script.lox>");
        process::exit(64);
    });

    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Can not read {}: {}", path, e);
        process::exit(66);
    });

    for token in scan_tokens(&source) {
        println!("{:>3} {:?} '{}'", token.line, token.token_type, token.lexeme);
    }

    if had_error() {
        process::exit(65);
    }
}
