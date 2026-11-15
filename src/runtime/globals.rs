use std::{rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::runtime::{environment::Environment, function::NativeFunction, object::LoxObject};

fn fun_clock(args: &[LoxObject], _env: &Environment) -> Result<LoxObject, String> {
    if !args.is_empty() {
        return Err(format!("Function 'clock' takes 0 argument, but found {}", args.len()));
    }

    let start = SystemTime::now();
    let millis = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_millis() as f64;

    Ok(LoxObject::Number(millis))
}

pub fn build_global() -> Environment {
    let globals = Environment::new();
    globals.define("clock".into(), LoxObject::NativeFunction(Rc::new(NativeFunction{
        callable: fun_clock
    })));

    globals
}
