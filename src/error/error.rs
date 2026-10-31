use std::process;

pub fn syntax_error(line: usize, msg: &str) -> ! {
    eprintln!("[line {}] SyntaxError: {}", line, msg);
    process::exit(65);
}
