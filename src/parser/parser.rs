use std::{iter::Peekable, slice::Iter};

use crate::{
    error::error::syntax_error_at, parser::ast::{BinaryOperator, Expr, Program, Statement, UnaryOperator}, utils::token::{Token, TokenType},
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
            TokenType::Var => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().token_type {
            TokenType::Print => self.print_statement(),
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'print'
        let expr = self.expression()?;
        self.expect_token(TokenType::SemiColon, "Expected ';' after statement.")?;
        Ok(Statement::Print {
            expression: Box::new(expr),
        })
    }

    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        self.tokens.next(); // consume 'var'
        let name = self.expect_identifier("Expected variable name.")?;
        let initializer = if self.peek().token_type == TokenType::Equal {
            self.tokens.next(); // consume the '='
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.expect_token(TokenType::SemiColon, "Expected ';'")?;
        Ok(Statement::VarDeclaration { name, initializer })
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.expect_token(TokenType::SemiColon, "Expected ';' after expression.")?;
        Ok(Statement::Expression {
            expression: Box::new(expr),
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // Do return a ParseError here if the left-hand side is errornous
        let expr = self.equality()?;

        if matches!(self.peek().token_type, TokenType::Equal) {
            let equal_token = self.tokens.next().unwrap(); // consume the '='
            // Do return a ParseError here if the right-hand side is errornous
            let value = self.assignment()?;

            // report an error if the left-hand side is not a valid assignment target
            // but do not return ParseError, just report
            // Ex: a + b = c => Parsed fine but meaningless, so it is reported only
            if let Expr::Variable { name, line } = expr {
                return Ok(Expr::Assignment { name, value: Box::new(value), line  })
            } else {
                syntax_error_at(equal_token, "Invalid assignment target.");
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
            _ => return self.primary(),
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

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek();
        let expression = match &token.token_type {
            TokenType::False => Expr::Literal(super::ast::Literal::Bool(false)),
            TokenType::True => Expr::Literal(super::ast::Literal::Bool(true)),
            TokenType::Nil => Expr::Literal(super::ast::Literal::Nil),
            TokenType::Number(n) => Expr::Literal(super::ast::Literal::Number(*n)),
            TokenType::String(s) => Expr::Literal(super::ast::Literal::Str(s.clone())),
            TokenType::Identifier(name) => Expr::Variable { name: name.clone(), line: token.line },
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
