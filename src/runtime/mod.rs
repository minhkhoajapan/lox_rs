pub mod interpreter;
mod object;
mod environment;

struct RuntimeError{
    line: usize,
    msg: String,
}
