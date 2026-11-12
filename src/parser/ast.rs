#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Box<Expr>,
    },

    Print {
        expression: Box<Expr>,
    },

    VarDeclaration {
        name: String,
        initializer: Option<Box<Expr>>,
    },

    Block {
        statements: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Assignment {
        name: String,
        value: Box<Expr>,
        line: usize,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        line: usize,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
        line: usize,
    },
    Variable {
        name: String,
        line: usize,
    },
}

#[derive(Debug)]
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
    Slash,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    Bang,
}
