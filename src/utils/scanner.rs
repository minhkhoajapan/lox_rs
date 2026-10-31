use std::{iter::{Peekable}, str::Chars};

use crate::{error::error::syntax_error,utils::token::{Token, TokenType::{self}}};

pub fn scan_tokens(source: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut scanner = Scanner {
        source: source.chars().peekable(),
        line: 1
    };

    loop {
        let next_token = scanner.scan_token();
        if next_token.token_type == TokenType::Eof {
            tokens.push(next_token);
            break;
        } else {
            tokens.push(next_token);
        }
    }

    tokens
}

struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    line: usize
}

impl<'a> Scanner<'a> {
    fn scan_token(self: &mut Scanner<'a>) -> Token {
        if let Some(c) = self.source.peek() {
            if c.is_whitespace() {
                // skip as many whitespace as possible
                self.skip_till(|x| x.is_whitespace());
                self.scan_token()
            } else if c.is_ascii_digit() { // beggining of a Number type
                self.scan_number()
            } else if c == &'"' { // beggining of a String type
                self.scan_string()
            } else if c.is_ascii_alphabetic() || c == &'_' { // beggining of a Keyword or Identifier
                self.scan_keyword_or_identifier()
            } else {
                self.scan_symbol()
            }
        } else {
            Token {
                token_type: TokenType::Eof,
                lexeme: String::new(),
                line: self.line
            }
        }
    }

    fn scan_number(self: &mut Scanner<'a>) -> Token {
        let mut lexeme = String::new();
        // read as much digit as possible
        while let Some(c) = self.source.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            lexeme.push(self.next().unwrap());
        }

        // if encounter a '.', add it and then read as much digit as possible
        if self.source.peek() == Some(&'.') 
            && self.peek_next().is_some_and(|c| c.is_ascii_digit()){
            lexeme.push(self.next().unwrap());

            while let Some(c) = self.source.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                lexeme.push(self.next().unwrap());
            }
        }
        
        Token {
            token_type: TokenType::Number(lexeme.parse().unwrap()),
            lexeme,
            line: self.line
        }
    }

    fn scan_string(self: &mut Scanner<'a>) -> Token {
        let mut lexeme = String::new();
        self.next();

        while let Some(c) = self.source.peek() {
            if c == &'"' {
                break;
            }
            if c == &'\n' {
                self.line += 1;
            }
            lexeme.push(self.next().unwrap());
        }

        if self.match_next(&'"') {
            self.next();
            Token { token_type: TokenType::String(lexeme.parse().unwrap()), lexeme, line: self.line }
        } else {
            syntax_error(self.line, "Unterminated string")
        }
        
    }

    fn scan_keyword_or_identifier(self: &mut Scanner<'a>) -> Token {
        let mut lexeme = String::new();
        
        while let Some(c) = self.source.peek() {
            if !c.is_ascii_alphanumeric() && c != &'_' {
                break;
            }
            lexeme.push(self.next().unwrap());
        }

        Token {
            token_type: match lexeme.as_str() {
                "and" => TokenType::And,
                "class" => TokenType::Class,
                "else" => TokenType::Else,
                "false" => TokenType::False,
                "for" => TokenType::For,
                "fun" => TokenType::Fun,
                "if" => TokenType::If,
                "nil" => TokenType::Nil,
                "or" => TokenType::Or,
                "print" => TokenType::Print,
                "return" => TokenType::Return,
                "super" => TokenType::Super,
                "this" => TokenType::This,
                "true" => TokenType::True,
                "var" => TokenType::Var,
                "while" => TokenType::While,
                _ => TokenType::Identifier(lexeme.clone())
            },
            lexeme,
            line: self.line
        }
    }

    fn scan_symbol(self: &mut Scanner<'a>) -> Token {
        let mut lexeme = String::new();

        let c = self.next().unwrap();
        lexeme.push(c);

        let token_type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::SemiColon,
            '*' => TokenType::Star,
            '!' => {
                if self.match_next(&'=') {
                    lexeme.push(self.next().unwrap());
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            },
            '=' => {
                if self.match_next(&'=') {
                    lexeme.push(self.next().unwrap());
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            },
            '<' => {
                if self.match_next(&'=') {
                    lexeme.push(self.next().unwrap());
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            },
            '>' => {
                if self.match_next(&'=') {
                    lexeme.push(self.next().unwrap());
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            },
            '/' => {
                if self.match_next(&'/') {
                    self.skip_till(|x| !x.eq(&'\n')); // 1 line comment so skip the whole line
                    return self.scan_token()
                } else {
                    TokenType::Slash
                }
            },
            _ => syntax_error(self.line, &format!("Unexpected character: '{}'", c)),
        };

        Token {
            token_type,
            lexeme,
            line: self.line
        }
    }
    
    // consume current character and move to the next
    fn next(self: &mut Scanner<'a>) -> Option<char> {
        let c = self.source.next()?;
        Some(c)
    }

    // peek 2 character ahead
    fn peek_next(self: &mut Scanner<'a>) -> Option<char> {
        let mut it = self.source.clone();
        it.next();
        it.next()
    }

    fn skip_till(self: &mut Scanner<'a>, till: impl Fn(&char) -> bool) {
        while let Some(c) = self.source.peek() {
            if !till(c) {
                break;
            }
            if c == &'\n' {
                self.line += 1;
            }
            self.next();
        }
    }

    fn match_next(self: &mut Scanner<'a>, target: &char) -> bool {
        if let Some(c) = self.source.peek() {
            if c == target {
                return true;           
            }
            return false
        }
        false
    }
}
