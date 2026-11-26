pub mod interpreter;
mod object;
mod class;
mod environment;
mod function;
mod globals;

struct RuntimeError{
    line: usize,
    msg: String,
}
