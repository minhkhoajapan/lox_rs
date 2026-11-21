use std::{fmt, rc::Rc};

use crate::{parser::ast::Statement, runtime::{environment::Environment, object::LoxObject}};

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
pub struct UserFunction{
    pub name: String,
    pub params: Rc<Vec<String>>,
    pub body: Rc<Statement>,
    pub closure: Environment,
}

impl UserFunction {
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

impl std::fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
