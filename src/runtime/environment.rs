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
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn get_at(&self, name: &str, depth: usize) -> Result<LoxObject, String> {
        if depth == 0 {
            return self.get(name);
        }

        let scope = self.0.borrow();
        match &scope.enclosing {
            Some(enclosing) => enclosing.get_at(name, depth - 1),
            None => unreachable!("resolver depth is deeper than the environment chain.")
        }
    }

    pub fn assign(&self, name: &str, value: LoxObject) -> Result<(), String> {
        let mut scope = self.0.borrow_mut();
        if let Some(slot) = scope.values.get_mut(name) {
            *slot = value;
            Ok(())
        }  else {
            Err(format!("Undefined variable '{}'", name))
        }
    }

    pub fn assign_at(&self, name: &str, value: LoxObject, depth: usize) -> Result<(), String> {
        if depth == 0 {
            return self.assign(name, value);
        }

        let scope = self.0.borrow();
        match &scope.enclosing {
            Some(enclosing) => enclosing.assign_at(name, value, depth - 1),
            None => unreachable!("resolver depth is deeper than the environment chain.")
        }
    }
}
