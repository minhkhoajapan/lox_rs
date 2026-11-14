use std::fmt;

#[derive(Debug, Clone)]
pub enum LoxObject {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl LoxObject {
    pub fn neg(&self) -> Result<LoxObject, &'static str> {
        match self {
            LoxObject::Number(num) => Ok(LoxObject::Number(-num)),
            _ => Err("Operand must be a Number."),
        }
    }

    pub fn not(&self) -> LoxObject {
        LoxObject::Bool(!self.is_truthy())
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, LoxObject::Nil | LoxObject::Bool(false))
    }

    pub fn eq(&self, other: &LoxObject) -> LoxObject {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Bool(x == y),
            (Str(x), Str(y)) => Bool( x == y),
            (Bool(x), Bool(y)) => Bool (x == y),
            (Nil, Nil) => Bool(true),
            _ => Bool(false)
        }
    }

    pub fn neq(&self, other: &LoxObject) -> LoxObject {
        self.eq(other).not()
    }

    pub fn gt(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Bool(x > y)),
            _ => Err("Operands must be Number"),
        }
    }

    pub fn ge(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Bool(x >= y)),
            _ => Err("Operands must be Number"),
        }
    }

    pub fn lt(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Bool(x < y)),
            _ => Err("Operands must be Number"),
        }
    }

    pub fn le(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Bool(x <= y)),
            _ => Err("Operands must be Number"),
        }
    }

    pub fn add(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Number(x + y)),
            (Str(x), Str(y)) => Ok(Str(format!("{}{}", x, y))),
            _ => Err("Operands must both be Number or Str")
        }
    }

    pub fn minus(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Number(x - y)),
            _ => Err("Operands must be Number")
        }
    }

    pub fn mul(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => Ok(Number(x * y)),
            _ => Err("Operands must be Number")
        }
    }

    pub fn div(&self, other: &LoxObject) -> Result<LoxObject, &'static str> {
        use LoxObject::*;
        match (self, other) {
            (Number(x), Number(y)) => {
                if *y == 0.0 {
                    Err("Can not divide by zero")
                } else {
                    Ok(Number(x / y))
                }
            },
            _ => Err("Operands must be Number")
        }
    }
} 

impl std::fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxObject::Number(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::Str(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::Bool(val) => f.write_fmt(format_args!("{}", val)),
            LoxObject::Nil => f.write_str("nil")
        }
    }
}
