use crate::{error::error::runtime_error, parser::ast::{BinaryOperator, Expr, Literal, Program, Statement, UnaryOperator}, runtime::{RuntimeError, object::LoxObject}};

pub struct Interpreter {
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Err(e) = self.exec_expression(stmt) {
                runtime_error(e.line, e.msg);
                return;
            }
        }
    }

    fn exec_expression(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression { expression } => {
                self.eval_expression(expression)?;
                Ok(())
            },
            Statement::Print { expression } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj);
                Ok(())
            }
        }
    }

    fn eval_expression(&mut self, expr: &Expr) -> Result<LoxObject, RuntimeError> {
        match expr {
            Expr::Grouping(expr) => self.eval_expression(expr),
            Expr::Literal(val) => self.eval_literal_value(val),
            Expr::Unary {operator, right, line } => self.eval_unary(operator, right, line),
            Expr::Binary { left, operator, right, line } => self.eval_binary(left, operator, right, line)
        }
    }

    fn eval_literal_value(&mut self, val: &Literal) -> Result<LoxObject, RuntimeError> {
        Ok(match val {
            Literal::Number(num) => LoxObject::Number(*num),
            Literal::Bool(b) => LoxObject::Bool(*b),
            Literal::Str(s) => LoxObject::Str(s.clone()),
            Literal::Nil => LoxObject::Nil,
        })
    }

    fn eval_unary(&mut self, operator: &UnaryOperator, right_expr: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let expr_value = self.eval_expression(right_expr)?;   
        match operator {
            UnaryOperator::Minus => expr_value.neg().map_err(|msg| RuntimeError{line: *line, msg}),
            UnaryOperator::Bang => Ok(expr_value.not()),
        }
    }

    fn eval_binary(&mut self, left: &Expr, operator: &BinaryOperator, right: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

        match operator {
            BinaryOperator::EqualEqual => Ok(left.eq(&right)),
            BinaryOperator::BangEqual => Ok(left.neq(&right)),
            BinaryOperator::Greater => left.gt(&right).map_err(|msg| RuntimeError { line: *line, msg }),
            BinaryOperator::GreaterEqual => left.ge(&right).map_err(|msg| RuntimeError { line: *line, msg }),
            BinaryOperator::Less => left.lt(&right).map_err(|msg| RuntimeError { line: *line, msg }),
            BinaryOperator::LessEqual => left.le(&right).map_err(|msg| RuntimeError { line: *line, msg }),
            BinaryOperator::Plus => left.add(&right).map_err(|msg| RuntimeError {line: *line, msg}),
            BinaryOperator::Minus => left.minus(&right).map_err(|msg| RuntimeError {line: *line, msg}),
            BinaryOperator::Star => left.mul(&right).map_err(|msg| RuntimeError {line: *line, msg}),
            BinaryOperator::Slash => left.div(&right).map_err(|msg| RuntimeError {line: *line, msg}),
        }
    }
}
