use std::fmt;

use crate::runtime::{environment::Environment, object::LoxObject};

#[derive(Debug)]
pub struct NativeFunction{
    pub callable: fn (&[LoxObject], &Environment) -> Result<LoxObject, String>,
}

impl std::fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

#[derive(Debug)]
pub struct UserFunction{}
