use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::error::runtime_error, parser::ast::{BinaryOperator, Expr, Literal, LogicalOperator, Program, Statement, UnaryOperator}, runtime::{RuntimeError, class::{LoxClass, LoxInstance}, environment::Environment, function::{UserFunction}, globals, object::LoxObject}};

enum Flow {
    Normal,
    Return(LoxObject),
}

pub struct Interpreter {
    pub environment: Environment,
    #[allow(dead_code)]
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

    fn exec_statement(&mut self, statement: &Statement) -> Result<Flow, RuntimeError> {
        match statement {
            Statement::Block { statements } => {
                self.exec_block(statements, Environment::with_enclosing(self.environment.clone()))
            },
            Statement::Class { name, methods, .. } => {
                self.exec_class_declaration(name, methods);
                Ok(Flow::Normal)
            },
            Statement::ConditionalExpression { condition, then_branch, else_branch } => {
                self.exec_conditional(condition, then_branch, else_branch)
            },
            Statement::Expression { expression } => {
                self.eval_expression(expression)?;
                Ok(Flow::Normal)
            },
            Statement::FunctionDeclaration { name, params, body, .. } => {
                self.exec_function_declaration(name, params, body)?;
                Ok(Flow::Normal)
            },
            Statement::Print { expression } => {
                let obj = self.eval_expression(expression)?;
                println!("{}", obj);
                Ok(Flow::Normal)
            },
            Statement::Return { expression, .. } => {
                let value = match expression {
                    Some(expr) => self.eval_expression(expr)?,
                    None => LoxObject::Nil
                };
                Ok(Flow::Return(value))
            },
            Statement::VarDeclaration { name, initializer, .. } => {
                self.exec_var_declaration(name, initializer)?;
                Ok(Flow::Normal)
            },
            Statement::WhileLoop { condition, body } => {
                self.exec_whileloop(condition, body)
            },
        }
    }

    fn exec_block(&mut self, statements: &[Statement], enclosing_env: Environment) -> Result<Flow, RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, enclosing_env);  // cheap Rc clone
        let mut result = Ok(Flow::Normal);
        for s in statements {
            result = self.exec_statement(s);
            if !matches!(result, Ok(Flow::Normal)) {
                break;
            }
        }
        self.environment = previous;

        result
     }

    fn exec_class_declaration(&mut self, name: &str, methods: &[Statement]) {
        let mut method_map = HashMap::new();
        for method in methods {
            let Statement::FunctionDeclaration { name, params, body, .. } = method else {
                unreachable!("class body only contains methods");
            };
            method_map.insert(name.clone(), Rc::new(UserFunction {
                name: name.clone(),
                params: Rc::clone(params),
                body: Rc::clone(body),
                closure: self.environment.clone(),
                is_initializer: name == "init",
            }));
        }

        let class = LoxClass { name: name.to_owned(), methods: method_map };
        self.environment.define(name.to_owned(), LoxObject::Class(Rc::new(class)));
    }

    fn exec_conditional(&mut self, condition: &Expr, then_branch: &Statement, else_branch: &Option<Box<Statement>>) -> Result<Flow, RuntimeError> {
        if self.eval_expression(condition)?.is_truthy() {
            self.exec_statement(then_branch)
        } else if let Some(else_branch_unwrapped) = else_branch {
            self.exec_statement(else_branch_unwrapped)
        } else {
            Ok(Flow::Normal)
        }
    }

    fn exec_function_declaration(&mut self, name: &str, params: &Rc<Vec<String>>, body: &Rc<Statement>) -> Result<(), RuntimeError> {
        let function = UserFunction {
            name: name.to_owned(),
            params: Rc::clone(params),
            body: Rc::clone(body),
            closure: self.environment.clone(),
            is_initializer: false,
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

    fn exec_whileloop(&mut self, condition: &Expr, body: &Statement) -> Result<Flow, RuntimeError> {
        while self.eval_expression(condition)?.is_truthy() {
            let flow = self.exec_statement(body)?;
            if let Flow::Return(_) = flow {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn eval_expression(&mut self, expr: &Expr) -> Result<LoxObject, RuntimeError> {
        match expr {
            Expr::Assignment { name, value, line, resolved_depth } => self.eval_assignment(name, value, line, resolved_depth),
            Expr::Binary { left, operator, right, line } => self.eval_binary(left, operator, right, line),
            Expr::Call { callee, arguments, line } => self.eval_call_expr(callee, arguments, line),
            Expr::Get { object, name, line } => self.eval_get(object, name, line),
            Expr::Grouping(expr) => self.eval_expression(expr),
            Expr::Literal(val) => self.eval_literal_value(val),
            Expr::Logical { left, operator, right } => self.eval_logical(left, operator, right),
            Expr::Set { object, name, value, line } => self.eval_set(object, name, value, line),
            Expr::This { line, resolved_depth } => self.eval_variable("this", line, resolved_depth),
            Expr::Unary {operator, right, line } => self.eval_unary(operator, right, line),
            Expr::Variable { name, line, resolved_depth } => self.eval_variable(name, line, resolved_depth),
        }
    }

    fn eval_assignment(&mut self, name: &str, value: &Expr, line: &usize, resolved_depth: &Option<usize>) -> Result<LoxObject, RuntimeError> {
        let value = self.eval_expression(value)?;
        if let Some(depth) = *resolved_depth {
            self.environment
                .assign_at(name, value.clone(), depth)
                .map_err(|msg| RuntimeError {line: *line, msg})?
        } else {
            self.globals
                .assign(name, value.clone())
                .map_err(|msg| RuntimeError {line: *line, msg})?
        }

        Ok(value)
    }

    fn eval_binary(&mut self, left: &Expr, operator: &BinaryOperator, right: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

        match operator {
            BinaryOperator::BangEqual => Ok(left.neq(&right)),
            BinaryOperator::EqualEqual => Ok(left.eq(&right)),
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
            LoxObject::Class(class) => self.call_class(&class, args, line),
            _ => Err(RuntimeError { line: *line, msg: "Can only call functions and classes".into() })
        }
    }

    fn call_class(&mut self, class: &Rc<LoxClass>, args: Vec<LoxObject>, line: &usize) -> Result<LoxObject, RuntimeError> {
        let instance = LoxObject::Instance(Rc::new(RefCell::new(LoxInstance {
            class: Rc::clone(class),
            fields: HashMap::new(),
        })));

        match class.methods.get("init") {
            Some(init) => {
                self.call_user_function(&init.bind(instance.clone()), args, line)?;
            },
            None if !args.is_empty() => {
                return Err(RuntimeError { line: *line, msg: format!("Expected 0 arguments but got {}.", args.len()) });
            },
            None => {},
        }
        Ok(instance)
    }

    fn call_user_function(&mut self, f: &UserFunction, args: Vec<LoxObject>, line: &usize) -> Result<LoxObject, RuntimeError> {
        if args.len() != f.arity() {
            return Err(RuntimeError { line: *line, msg: format!("Expected {} arguments but got {}.", f.arity(), args.len()) });
        }

        let enclosing_env = Environment::with_enclosing(f.closure.clone());
        for (param, arg) in f.params.iter().zip(args) {
            enclosing_env.define(param.clone(), arg);
        }

        let Statement::Block { statements } = f.body.as_ref() else {
            unreachable!("function body is always a block");
        };

        let flow = self.exec_block(statements, enclosing_env)?;

        // init() always returns 'this', even on a bare 'return;'
        if f.is_initializer {
            return f.closure.get_at("this", 0).map_err(|msg| RuntimeError { line: *line, msg });
        }

        match flow {
            Flow::Normal => Ok(LoxObject::Nil),
            Flow::Return(value) => Ok(value)
        }
    }

    fn eval_get(&mut self, object: &Expr, name: &str, line: &usize) -> Result<LoxObject, RuntimeError> {
        let LoxObject::Instance(instance) = self.eval_expression(object)? else {
            return Err(RuntimeError { line: *line, msg: "Only instances have properties.".into() });
        };

        let inst = instance.borrow();
        // fields shadow methods
        if let Some(value) = inst.fields.get(name) {
            return Ok(value.clone());
        }
        if let Some(method) = inst.class.methods.get(name) {
            let bound = method.bind(LoxObject::Instance(Rc::clone(&instance)));
            return Ok(LoxObject::UserFunction(Rc::new(bound)));
        }
        Err(RuntimeError { line: *line, msg: format!("Undefined property '{}'.", name) })
    }

    fn eval_set(&mut self, object: &Expr, name: &str, value: &Expr, line: &usize) -> Result<LoxObject, RuntimeError> {
        let LoxObject::Instance(instance) = self.eval_expression(object)? else {
            return Err(RuntimeError { line: *line, msg: "Only instances have fields.".into() });
        };
        let value = self.eval_expression(value)?;
        instance.borrow_mut().fields.insert(name.to_owned(), value.clone());
        Ok(value)
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

    fn eval_variable(&mut self, name: &str, line: &usize, resolved_depth: &Option<usize>) -> Result<LoxObject, RuntimeError> {
        let obj_ref = if let Some(depth) = *resolved_depth {
            self.environment
                .get_at(name, depth)
                .map_err(|msg| RuntimeError {line: *line, msg})?
        } else {
            self.globals
                .get(name)
                .map_err(|msg| RuntimeError { line: *line, msg })?
        };

        Ok(obj_ref)
    }
}
