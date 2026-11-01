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
        operator: BinaryOperator,
        right: Box<Expr>
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>
    },
}

pub enum BinaryOperator {
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash
}

pub enum UnaryOperator {
    Minus,
    Bang
}


