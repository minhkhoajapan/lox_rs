use std::{cell::RefCell, collections::HashMap, fmt::format, rc::Rc};
use crate::runtime::{object::LoxObject};

#[derive(Debug)]
pub struct Environment(Rc<RefCell<EnvironmentScope>>);

#[derive(Debug)]
struct EnvironmentScope {
    enclosing: Option<Environment>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope { 
            enclosing: None, 
            values: HashMap::new(),
        })))
    }

    pub fn define(&mut self, name: String, value: LoxObject) {
        self.0.borrow_mut().values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<LoxObject, String> {
        if let Some(value) = self.0.borrow().values.get(name) {
            Ok(value.clone())
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn assign(&mut self, name: &str, value: LoxObject) -> Result<(), String> {
        match self.0.borrow_mut().values.get_mut(name) {
            Some(slot) => {
                *slot = value;
                Ok(())
            },
            None => {
                Err(format!("Undefined variable '{}'", name))
            }
        }
    }
}
