use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::runtime::{object::LoxObject};

#[derive(Debug, Clone)]
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

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentScope { 
            enclosing: Some(enclosing), 
            values: HashMap::new() 
        })))
    }

    pub fn define(&self, name: String, value: LoxObject) {
        self.0.borrow_mut().values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<LoxObject, String> {
        let scope = self.0.borrow();
        if let Some(value) = scope.values.get(name) {
            Ok(value.clone())
        }
        else if let Some(enclose) = &scope.enclosing {
            enclose.get(name)
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn assign(&self, name: &str, value: LoxObject) -> Result<(), String> {
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
