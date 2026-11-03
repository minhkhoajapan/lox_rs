mod object;
pub mod interpreter;

struct RuntimeError{
    line: usize,
    msg: &'static str,
}
