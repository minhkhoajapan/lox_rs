use crate::utils::token::Token;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>
    },
}


