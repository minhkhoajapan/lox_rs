use std::{iter::Peekable, rc::Rc, slice::Iter};

use crate::{
    error::error::syntax_error_at, parser::ast::{BinaryOperator, Expr, LogicalOperator, Program, Statement, UnaryOperator}, utils::token::{Token, TokenType},
};

#[derive(Debug)]
struct ParseError;

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    // the error is already reported by syntax_error_at, so callers only need to know if it worked
    pub fn parse(&mut self) -> Program {
        let mut program = Program {statements: Vec::new()};
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => program.statements.push(stmt),
                _ => {
                    self.synchronize();
                }
            }
        }

        program
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::SemiColon => {
                    self.tokens.next(); // consume the ';' and return
                    return;
                },
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                },

                _ => {
                    self.tokens.next();
                }
            }
        }
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        match self.peek().token_type {
            TokenType::Class => self.class_declaration(),
            TokenType::Fun => {
                self.tokens.next(); // consume 'fun'
                self.function_declaration()
            },
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn class_declaration(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'class'
        let line = self.peek().line;
        let name = self.expect_identifier("Expect class name.")?;
        self.expect_token(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function_declaration()?);
        }
        self.expect_token(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Statement::Class { name, methods, line })
    }

    // parses a function or a method, starting at its name ('fun' is already consumed for functions)
    fn function_declaration(&mut self) -> Result<Statement, ParseError> {
        let line = self.peek().line;
        let name = self.expect_identifier("Expect function name.")?;
        self.expect_token(TokenType::LeftParen, "Expect '(' after function name.")?;

        let mut params = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    syntax_error_at(self.peek(), "Can't have more than 255 parameters.");
                    return Err(ParseError)
                }
                
                params.push(self.expect_identifier("Expect parameter name.")?);

                if matches!(self.peek().token_type, TokenType::Comma) {
                    self.tokens.next(); //consume ','
                } else {
                    break;
                }
            }
        }
        self.expect_token(TokenType::RightParen, "Expect ')' after parameters.")?;

        // do not consume '{' here because block_statement() will consume it later
        if !matches!(self.peek().token_type, TokenType::LeftBrace) {
            syntax_error_at(self.peek(), "Expect '{' before function body.");
            return Err(ParseError)
        }

        let body = self.block_statement()?;

        Ok(Statement::FunctionDeclaration { name, params: Rc::from(params), body: Rc::from(body), line })
    }

    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'var'
        let line = self.peek().line;
        let name = self.expect_identifier("Expected variable name.")?;
        let initializer = if self.peek().token_type == TokenType::Equal {
            self.tokens.next(); // consume the '='
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.expect_token(TokenType::SemiColon, "Expected ';'")?;
        Ok(Statement::VarDeclaration { name, initializer, line })
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().token_type {
            TokenType::For => self.for_statement(),
            TokenType::If => self.if_statement(),
            TokenType::Print => self.print_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::While => self.while_statemen(),
            TokenType::LeftBrace => self.block_statement(),
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'for'
        self.expect_token(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        
        let initializer = match self.peek().token_type {
            TokenType::SemiColon => {
                self.tokens.next(); // consume the ';'
                None
            },
            TokenType::Var => Some(self.var_declaration()?),
            _ => Some(self.expression_statement()?),
        };

        let condition = if self.peek().token_type != TokenType::SemiColon {
            Some(self.expression()?)  
        } else {
            None
        };
        self.expect_token(TokenType::SemiColon, "Expect ';' after for clauses.")?;

        let increment = if self.peek().token_type != TokenType::RightParen {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect_token(TokenType::RightParen, "Expect ')' after for clauses.")?;


        // desugaring into while loop
        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Statement::Block { statements: vec![body, Statement::Expression { expression: Box::new(increment) }] }
        };

        let condition = condition.unwrap_or(Expr::Literal(super::ast::Literal::Bool(true)));
        body = Statement::WhileLoop { condition: Box::new(condition), body: Box::new(body)};

        if let Some(initializer) = initializer {
            body = Statement::Block { statements: vec![initializer, body] };
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'if'
        self.expect_token(TokenType::LeftParen, "Expect '(' after 'if'.")?; // consume "("
        let condition = Box::new(self.expression()?);
        self.expect_token(TokenType::RightParen, "Expect ')' after if condition.")?; // consume ")"
        let then_branch = Box::new(self.statement()?);
        let else_branch = if matches!(self.peek().token_type, TokenType::Else) {
            self.tokens.next(); // consume "else"
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Statement::ConditionalExpression { condition, then_branch, else_branch })
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'print'
        let expr = self.expression()?;
        self.expect_token(TokenType::SemiColon, "Expected ';' after statement.")?;
        Ok(Statement::Print {
            expression: Box::new(expr),
        })
    }

    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let line = self.peek().line;
        self.tokens.next(); // consume 'return'
        if matches!(self.peek().token_type, TokenType::SemiColon) {
            self.tokens.next(); // consume the ';'
            return Ok(Statement::Return { expression: None, line })
        }

        let expr = Box::new(self.expression()?);
        self.expect_token(TokenType::SemiColon, "Expect ';' after return value.")?;
        Ok(Statement::Return { expression: Some(expr), line })
    }

    fn while_statemen(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consune 'while'
        self.expect_token(TokenType::LeftParen, "Expect '(' after while.")?;
        let condition = self.expression()?;
        self.expect_token(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Statement::WhileLoop { condition: Box::new(condition), body: Box::new(body) })
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.expect_token(TokenType::SemiColon, "Expected ';' after expression.")?;
        Ok(Statement::Expression {
            expression: Box::new(expr),
        })
    }

    fn block_statement(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume '{'
        let mut statements = Vec::new();
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                _ => {
                    self.synchronize();
                }
            }
        }
        self.expect_token(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(Statement::Block { statements })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // Do return a ParseError here if the left-hand side is errornous
        let expr = self.or()?;

        if matches!(self.peek().token_type, TokenType::Equal) {
            let equal_token = self.tokens.next().unwrap(); // consume the '='
            // Do return a ParseError here if the right-hand side is errornous
            let value = self.assignment()?;

            // report an error if the left-hand side is not a valid assignment target
            // but do not return ParseError, just report
            // Ex: a + b = c => Parsed fine but meaningless, so it is reported only
            match expr {
                Expr::Variable { name, line, .. } => {
                    return Ok(Expr::Assignment { name, value: Box::new(value), line, resolved_depth: None  })
                },
                Expr::Get { object, name, line } => {
                    return Ok(Expr::Set { object, name, value: Box::new(value), line })
                },
                _ => syntax_error_at(equal_token, "Invalid assignment target."),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        loop {
            let logical_operator = match &self.peek().token_type {
                TokenType::Or => LogicalOperator::Or,
                _ => break,
            };
            self.tokens.next(); // consume the "or"
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: logical_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        loop {
            let logical_operator = match &self.peek().token_type {
                TokenType::And => LogicalOperator::And,
                _ => break,
            };
            self.tokens.next(); // consime the "and"
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: logical_operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparision()?;
        loop {
            let binary_operator = match &self.peek().token_type {
                TokenType::EqualEqual => BinaryOperator::EqualEqual,
                TokenType::BangEqual => BinaryOperator::BangEqual,
                _ => break,
            };
            let line = self.peek().line;
            self.tokens.next();
            let right = self.comparision()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
                line,
            }
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        loop {
            let binary_operator = match &self.peek().token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                _ => break,
            };
            let line = self.peek().line;
            self.tokens.next();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
                line,
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        loop {
            let binary_operator = match &self.peek().token_type {
                TokenType::Plus => BinaryOperator::Plus,
                TokenType::Minus => BinaryOperator::Minus,
                _ => break,
            };
            let line = self.peek().line;
            self.tokens.next();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
                line,
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        loop {
            let binary_operator = match &self.peek().token_type {
                TokenType::Slash => BinaryOperator::Slash,
                TokenType::Star => BinaryOperator::Star,
                _ => break,
            };
            let line = self.peek().line;
            self.tokens.next();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: binary_operator,
                right: Box::new(right),
                line,
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let unary_operator = match &self.peek().token_type {
            TokenType::Bang => UnaryOperator::Bang,
            TokenType::Minus => UnaryOperator::Minus,
            _ => return self.call(),
        };
        let line = self.peek().line;
        self.tokens.next();
        let expr = self.unary()?;
        Ok(Expr::Unary {
            operator: unary_operator,
            right: Box::new(expr),
            line,
        })
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            expr = match self.peek().token_type {
                TokenType::LeftParen => self.finish_call(Box::new(expr))?,
                TokenType::Dot => {
                    self.tokens.next(); // consume '.'
                    let line = self.peek().line;
                    let name = self.expect_identifier("Expect property name after '.'.")?;
                    Expr::Get { object: Box::new(expr), name, line }
                },
                _ => break
            }
        }
        
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> Result<Expr, ParseError> {
        self.tokens.next(); // consume the '('
        let mut args = Vec::new();
        if !matches!(self.peek().token_type, TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    syntax_error_at(self.peek(), "Can't have more than 255 arguments");
                    return Err(ParseError);
                }
                args.push(self.expression()?);

                if matches!(self.peek().token_type, TokenType::Comma) {
                    self.tokens.next(); // consume the ','
                } else {
                    break;
                }
            }
        } 
        let line = self.peek().line;
        self.expect_token(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call { callee, arguments: args, line })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek();
        let expression = match &token.token_type {
            TokenType::False => Expr::Literal(super::ast::Literal::Bool(false)),
            TokenType::True => Expr::Literal(super::ast::Literal::Bool(true)),
            TokenType::Nil => Expr::Literal(super::ast::Literal::Nil),
            TokenType::Number(n) => Expr::Literal(super::ast::Literal::Number(*n)),
            TokenType::String(s) => Expr::Literal(super::ast::Literal::Str(s.clone())),
            TokenType::Identifier(name) => Expr::Variable { name: name.clone(), line: token.line, resolved_depth: None },
            TokenType::This => Expr::This { line: token.line, resolved_depth: None },
            _ => {
                return self.grouping();
            }
        };

        self.tokens.next();

        Ok(expression)
    }

    fn peek(&mut self) -> &'a Token {
        self.tokens.peek().unwrap()
    }

    fn grouping(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek().token_type, TokenType::LeftParen) {
            self.tokens.next();
            let expr = self.expression()?;
            if matches!(self.peek().token_type, TokenType::RightParen) {
                self.tokens.next();
                Ok(Expr::Grouping(Box::new(expr)))
            } else {
                syntax_error_at(self.peek(), "Expected ')'.");
                Err(ParseError)
            }
        } else {
            syntax_error_at(self.peek(), "Expected grouping.");
            Err(ParseError)
        }
    }

    // check if the token is of a certain kind.
    // if true then consume and return that token
    // else print out the error and return ParseError
    fn expect_token(&mut self, expected: TokenType, error_msg: &str) -> Result<&Token, ParseError> {
        if self.peek().token_type == expected {
            Ok(self.tokens.next().unwrap())
        } else {
            syntax_error_at(self.peek(), error_msg);
            Err(ParseError)
        }
    }

    // check if the token is of type identifier
    // if true then consume and return the String that is contained in that token
    // else print out the error and return ParseError
    fn expect_identifier(&mut self, msg: &str) -> Result<String, ParseError> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.tokens.next(); // consume the token
            Ok(name)
        } else {
            syntax_error_at(self.peek(), msg);
            Err(ParseError)
        }
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }
}
