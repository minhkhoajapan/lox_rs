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
    pub is_initializer: bool,
}

impl UserFunction {
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    // wrap the closure in a new scope that only holds 'this'
    pub fn bind(&self, instance: LoxObject) -> UserFunction {
        let env = Environment::with_enclosing(self.closure.clone());
        env.define("this".into(), instance);
        UserFunction {
            name: self.name.clone(),
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: env,
            is_initializer: self.is_initializer,
        }
    }
}

impl std::fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
