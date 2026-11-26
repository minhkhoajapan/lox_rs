use std::{collections::HashMap, rc::Rc};

use crate::{error::error::resolve_error, parser::ast::{Expr, Program, Statement},};

struct ResolveError {
    line: usize,
    msg: String,
}

#[derive(Clone, Copy, Debug)]
enum DeclarationState {
    Declared,
    Defined,
}

#[derive(Clone, Copy, Debug)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, Debug)]
enum ClassType {
    None,
    Class,
}

struct Resolver {
    scopes: Vec<HashMap<String, DeclarationState>>,
    current_function_type: FunctionType,
    current_class_type: ClassType,
}

pub fn resolve_program(program: &mut Program) {
    let mut resolver = Resolver::new();
    resolver.resolve_program(program);
}

impl Resolver {
    fn resolve_program(&mut self, program: &mut Program) {
        for stmt in program.statements.iter_mut() {
            if let Err(e) = self.resolve_statement(stmt) {
                resolve_error(e.line, &e.msg);
            }
        }
    }

    fn new() -> Self {
        Resolver { 
            scopes: Vec::new(), 
            current_function_type: FunctionType::None,
            current_class_type: ClassType::None,
        }
    }

    fn resolve_statement(&mut self, stmt: &mut Statement) -> Result<(), ResolveError> {
        match stmt {
            Statement::Block { statements } => {
                self.resolve_block_statement(statements);
                Ok(())
            },
            Statement::Class { name, methods, line } => {
                self.resolve_class_declaration(name, methods, line);
                Ok(())
            },
            Statement::VarDeclaration { name, initializer, line } => {
                self.resolve_var_declaration(name, initializer, line);
                Ok(())
            },
            Statement::FunctionDeclaration { name, params, body, line } => {
                self.resolve_function_declaration(name, params, body, line);
                Ok(())
            },
            Statement::Expression { expression } => self.resolve_expression(expression),
            Statement::Print { expression } => self.resolve_expression(expression),
            Statement::ConditionalExpression { condition, then_branch, else_branch } => {
                self.resolve_conditional(condition, then_branch, else_branch);
                Ok(())
            },
            Statement::WhileLoop { condition, body } => {
                self.resolve_whileloop(condition, body);
                Ok(())
            },
            Statement::Return { expression, line } => {
                self.resolve_return_statement(expression, line)
            }
        }
    } 

    fn resolve_block_statement(&mut self, statements: &mut[Statement]) {
        self.begin_scope();
        for stmt in statements.iter_mut() {
            if let Err(e) = self.resolve_statement(stmt) {
                resolve_error(e.line, &e.msg);
            }
        }
        self.end_scope();
    }

    fn resolve_class_declaration(&mut self, name: &str, methods: &mut [Statement], line: &usize) {
        if let Err(e) = self.declare(name, line) {
            resolve_error(e.line, &e.msg);
        }
        self.define(name);

        let enclosing_class_type = self.current_class_type;
        self.current_class_type = ClassType::Class;

        // scope that holds 'this', matching the env created by UserFunction::bind
        self.begin_scope();
        self.define("this");
        for method in methods.iter_mut() {
            let Statement::FunctionDeclaration { name, params, body, line } = method else {
                unreachable!("class body only contains methods");
            };
            let function_type = if name == "init" { FunctionType::Initializer } else { FunctionType::Method };
            self.resolve_function(params, body, line, function_type);
        }
        self.end_scope();

        self.current_class_type = enclosing_class_type;
    }

    fn resolve_var_declaration(&mut self, name: &str, initializer: &mut Option<Box<Expr>>, line: &usize) {
        if let Err(e) = self.declare(name, line) {
            resolve_error(e.line, &e.msg);
        }
        if let Some(expr) = initializer 
          && let Err(e) = self.resolve_expression(expr){
            resolve_error(e.line, &e.msg);
        }
        self.define(name);
    }

    fn resolve_function_declaration(&mut self, name: &str, params: &Rc<Vec<String>>, body: &mut Rc<Statement>, line: &usize) {
        if let Err(e) = self.declare(name, line) {
            resolve_error(e.line, &e.msg);
        }
        self.define(name);
        self.resolve_function(params, body, line, FunctionType::Function);
    }

    // resolves params and body only; methods use this directly since their name isn't declared in a scope
    fn resolve_function(&mut self, params: &Rc<Vec<String>>, body: &mut Rc<Statement>, line: &usize, function_type: FunctionType) {
        let body = Rc::get_mut(body).expect("function body is shared before resolving");                                
        let Statement::Block { statements } = body else {
            unreachable!("function body is always a block");
        };

        let enclosing_function_type = self.current_function_type;
        self.current_function_type = function_type;
        self.begin_scope();
        for param in params.iter() {
            if let Err(e) = self.declare(param, line) {
                resolve_error(e.line, &e.msg);
            }
            self.define(param);
        }
        for stmt in statements.iter_mut() {
            if let Err(e) = self.resolve_statement(stmt) {
                resolve_error(e.line, &e.msg);
            }
        }
        self.end_scope();
        self.current_function_type = enclosing_function_type;
    }

    fn resolve_conditional(&mut self, condition: &mut Expr, then_branch: &mut Statement, else_branch: &mut Option<Box<Statement>>) {
        if let Err(e) = self.resolve_expression(condition) {
            resolve_error(e.line, &e.msg);
        }
        if let Err(e) = self.resolve_statement(then_branch) {
            resolve_error(e.line, &e.msg);
        }
        if let Some(stmt) = else_branch 
          && let Err(e) = self.resolve_statement(stmt) {
            resolve_error(e.line, &e.msg);
        }
    }

    fn resolve_whileloop(&mut self, condition: &mut Expr, body: &mut Statement) {
        if let Err(e) = self.resolve_expression(condition) {
            resolve_error(e.line, &e.msg);
        }

        if let Err(e) = self.resolve_statement(body) {
            resolve_error(e.line, &e.msg);
        }
    }

    fn resolve_return_statement(&mut self, expr: &mut Option<Box<Expr>>, line: &usize) -> Result<(), ResolveError> {
        if matches!(self.current_function_type, FunctionType::None) {
            Err(ResolveError { line: *line, msg: "Error at 'return': Can't return from a top level code".to_string() })
        } else if matches!(self.current_function_type, FunctionType::Initializer) && expr.is_some() {
            Err(ResolveError { line: *line, msg: "Error at 'return': Can't return a value from an initializer.".to_string() })
        } else if let Some(expr) = expr {
            self.resolve_expression(expr)
        } else {
            Ok(())
        }
    }

    fn resolve_expression(&mut self, expr: &mut Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Assignment { name, value, resolved_depth, .. } => {
                self.resolve_assignment_expression(name, value, resolved_depth);
                Ok(())
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            },
            Expr::Call { callee, arguments, .. } => {
                self.resolve_expression(callee)?;
                for arg in arguments.iter_mut() {
                    self.resolve_expression(arg)?;
                }
                Ok(())
            },
            Expr::Get { object, .. } => self.resolve_expression(object),
            Expr::Grouping(expr) => self.resolve_expression(expr),
            Expr::Literal(..) => Ok(()),
            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            },
            Expr::Set { object, value, .. } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)
            },
            Expr::This { line, resolved_depth } => {
                if matches!(self.current_class_type, ClassType::None) {
                    return Err(ResolveError { line: *line, msg: "Error at 'this': Can't use 'this' outside of a class.".to_string() });
                }
                self.resolve_local("this", resolved_depth);
                Ok(())
            },
            Expr::Unary { right, .. } => self.resolve_expression(right),
            Expr::Variable { name, line, resolved_depth } => self.resolve_variable_expression(name, line, resolved_depth)
        }
    }

    fn resolve_assignment_expression(&mut self, name: &str, expr: &mut Expr, resolved_depth: &mut Option<usize>) {
        if let Err(e) = self.resolve_expression(expr) {
            resolve_error(e.line, &e.msg);
        }
        self.resolve_local(name, resolved_depth);
    }

    fn resolve_variable_expression(&self, name: &str, line: &usize, resolved_depth: &mut Option<usize>) -> Result<(), ResolveError> {
        if !self.scopes.is_empty() && 
          matches!(self.get_state_in_current_scope(name), 
                   Some(DeclarationState::Declared)) {
            return Err(ResolveError { 
                line: *line, 
                msg: "Can't read local variable in its own initializer.".to_string(),
            })
        }

        self.resolve_local(name, resolved_depth);
        Ok(())
    }
    
    fn declare(&mut self, name: &str, line: &usize) -> Result<(), ResolveError> {
        if matches!(self.get_state_in_current_scope(name), Some(..)) {
            return Err(ResolveError { 
                line: *line, 
                msg: format!("Error at '{}': A variable with this name already exists in this scope ", name) 
            });
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), DeclarationState::Declared);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), DeclarationState::Defined);
        }
    }

    fn get_state_in_current_scope(&self, name: &str) -> Option<DeclarationState> {
        if let Some(scope) = self.scopes.last() {
            scope.get(name).cloned()
        } else {
            None
        }
    }

    fn resolve_local(&self, name: &str, resolved_depth: &mut Option<usize>) {
        *resolved_depth = self.scopes.iter().rev().position(|scope| scope.contains_key(name))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
