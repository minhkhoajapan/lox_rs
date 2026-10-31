use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::token::{Token, TokenType};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

pub fn syntax_error(line: usize, msg: &str) {
    report(line, "", msg);
}

// for the parser: also says which token the error is at
pub fn syntax_error_at(token: &Token, msg: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", msg);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), msg);
    }
}

pub fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}

fn report(line: usize, location: &str, msg: &str) {
    eprintln!("[line {}] SyntaxError{}: {}", line, location, msg);
    HAD_ERROR.store(true, Ordering::Relaxed);
}
