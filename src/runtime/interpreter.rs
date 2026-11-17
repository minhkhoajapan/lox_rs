use std::rc::Rc;

use crate::{error::error::runtime_error, parser::ast::{BinaryOperator, Expr, Literal, LogicalOperator, Program, Statement, UnaryOperator}, runtime::{RuntimeError, environment::Environment, function::{UserFunction}, globals, object::LoxObject}};

pub struct Interpreter {
    pub environment: Environment,
    pub globals: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let global_environment = globals::build_global();
        Interpreter {
            environment: global_environment.clone(),
            globals: global_environment.clone(),
        }
    }

    pub fn interpret(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Err(e) = self.exec_statement(stmt) {
                runtime_error(e.line, &e.msg);
                return;
            }
        }
    }

    fn exec_statement(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression { expression } => {
                self.eval_expression(expression)?;
                Ok(())
            },
            Statement::ConditionalExpression { condition, then_branch, else_branch } => {
                self.exec_conditional(condition, then_branch, else_branch)?;
                Ok(())
            },
            Statement::FunctionDeclaration { name, params, body } => {
                self.exec_function_declaration(name, params, body)?;
                Ok(())
            }
            Statement::Print { expression } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj);
                Ok(())
            },
            Statement::VarDeclaration { name, initializer } => {
                self.exec_var_declaration(name, initializer)?;
                Ok(())
            },
            Statement::WhileLoop { condition, body } => {
                self.exec_whileloop(condition, body)?;
                Ok(())
            },
            Statement::Block { statements } => {
                self.exec_block(statements, self.environment.clone())?;
                Ok(())
            },
        }
    }

    fn exec_function_declaration(&mut self, name: &str, params: &Rc<Vec<String>>, body: &Rc<Statement>) -> Result<(), RuntimeError> {
        let function = UserFunction {
            name: name.to_owned(),
            params: Rc::clone(params),
            body: Rc::clone(body),
        };

        self.environment.define(name.to_owned(), LoxObject::UserFunction(
            Rc::new(function)
        ));
        Ok(())
    }

    fn exec_var_declaration(&mut self, name: &str, initializer: &Option<Box<Expr>>) -> Result<(), RuntimeError> {
        let value = if let Some(expression) = initializer {
            self.eval_expression(expression)?
        } else {
            LoxObject::Nil
        };

        self.environment.define(name.to_owned(), value);
        Ok(())
    }

    fn exec_whileloop(&mut self, condition: &Expr, body: &Statement) -> Result<(), RuntimeError> {
        while self.eval_expression(condition)?.is_truthy() {
            self.exec_statement(body)?;
        }
        Ok(())
    }

    fn exec_block(&mut self, statements: &[Statement], enclosing_env: Environment) -> Result<(), RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, enclosing_env);  // cheap Rc clone
        let result = statements.iter().try_for_each(|s| self.exec_statement(s));
        self.environment = previous;

        result
     }

    fn eval_expression(&mut self, expr: &Expr) -> Result<LoxObject, RuntimeError> {
        match expr {
            Expr::Grouping(expr) => self.eval_expression(expr),
            Expr::Literal(val) => self.eval_literal_value(val),
            Expr::Logical { left, operator, right } => self.eval_logical(left, operator, right),
            Expr::Unary {operator, right, line } => self.eval_unary(operator, right, line),
            Expr::Binary { left, operator, right, line } => self.eval_binary(left, operator, right, line),
            Expr::Call { callee, arguments, line } => self.eval_call_expr(callee, arguments, line),
            Expr::Variable { name, line } => self.eval_variable(name, line),
            Expr::Assignment { name, value, line } => self.eval_assignment(name, value, line),
        }
    }

    fn exec_conditional(&mut self, condition: &Expr, then_branch: &Statement, else_branch: &Option<Box<Statement>>) -> Result<(), RuntimeError> {
        if self.eval_expression(condition)?.is_truthy() {
            self.exec_statement(then_branch)
        } else if let Some(else_branch_unwrapped) = else_branch {
            self.exec_statement(else_branch_unwrapped)
        } else {
            Ok(())
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

    fn eval_logical(&mut self, left: &Expr, operator: &LogicalOperator, right: &Expr) -> Result<LoxObject, RuntimeError> {
        let left_result = self.eval_expression(left)?;
        match operator {
            LogicalOperator::Or => {
                if left_result.is_truthy() {
                    return Ok(left_result);
                }
            },
            LogicalOperator::And => {
                if !left_result.is_truthy() {
                    return Ok(left_result);
                }
            }
        }
        self.eval_expression(right)
    }

    fn eval_unary(&mut self, operator: &UnaryOperator, right_expr: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let expr_value = self.eval_expression(right_expr)?;   
        match operator {
            UnaryOperator::Minus => expr_value.neg().map_err(|msg| RuntimeError{line: *line, msg: msg.into()}),
            UnaryOperator::Bang => Ok(expr_value.not()),
        }
    }

    fn eval_binary(&mut self, left: &Expr, operator: &BinaryOperator, right: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

        match operator {
            BinaryOperator::EqualEqual => Ok(left.eq(&right)),
            BinaryOperator::BangEqual => Ok(left.neq(&right)),
            BinaryOperator::Greater => left.gt(&right).map_err(|msg| RuntimeError { line: *line, msg: msg.into() }),
            BinaryOperator::GreaterEqual => left.ge(&right).map_err(|msg| RuntimeError { line: *line, msg: msg.into() }),
            BinaryOperator::Less => left.lt(&right).map_err(|msg| RuntimeError { line: *line, msg: msg.into() }),
            BinaryOperator::LessEqual => left.le(&right).map_err(|msg| RuntimeError { line: *line, msg: msg.into() }),
            BinaryOperator::Plus => left.add(&right).map_err(|msg| RuntimeError {line: *line, msg: msg.into()}),
            BinaryOperator::Minus => left.minus(&right).map_err(|msg| RuntimeError {line: *line, msg: msg.into()}),
            BinaryOperator::Star => left.mul(&right).map_err(|msg| RuntimeError {line: *line, msg: msg.into()}),
            BinaryOperator::Slash => left.div(&right).map_err(|msg| RuntimeError {line: *line, msg: msg.into()}),
        }
    }

    fn eval_call_expr(&mut self, callee: &Expr, args: &[Expr], line: &usize) -> Result<LoxObject, RuntimeError> {
        let callee = self.eval_expression(callee)?;
        let args = args
            .iter()
            .map(|a| self.eval_expression(a))
            .collect::<Result<Vec<_>, _>>()?;

        match callee {
            LoxObject::NativeFunction(f) => (f.callable)(&args, &self.environment).map_err(|msg| RuntimeError { line: *line, msg }),
            LoxObject::UserFunction(f) => self.call_user_function(&f, args, line),
            _ => Err(RuntimeError { line: *line, msg: "Can only call functions and classes".into() })
        }
    }

    fn call_user_function(&mut self, f: &UserFunction, args: Vec<LoxObject>, line: &usize) -> Result<LoxObject, RuntimeError> {
        if args.len() != f.arity() {
            return Err(RuntimeError { line: *line, msg: format!("Expected {} arguments but got {}.", f.arity(), args.len()) });
        }

        let enclosing_env = Environment::with_enclosing(self.globals.clone());
        for (param, arg) in f.params.iter().zip(args) {
            enclosing_env.define(param.clone(), arg);
        }

        let Statement::Block { statements } = f.body.as_ref() else {
            unreachable!("function body is always a block");
        };

        self.exec_block(statements, enclosing_env)?;
        Ok(LoxObject::Nil)
    }

    fn eval_variable(&mut self, name: &str, line: &usize) -> Result<LoxObject, RuntimeError> {
        self.environment.get(name).map_err(|msg| RuntimeError {line: *line, msg})
    }

    fn eval_assignment(&mut self, name: &str, value: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let value = self.eval_expression(value)?;
        self.environment.assign(name, value.clone()).map_err(|msg| RuntimeError {line: *line, msg})?;

        Ok(value)
    }
}
