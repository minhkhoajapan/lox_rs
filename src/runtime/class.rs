use std::{collections::HashMap, fmt, rc::Rc};

use crate::runtime::{function::UserFunction, object::LoxObject};

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Rc<UserFunction>>,
}

impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, LoxObject>,
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
