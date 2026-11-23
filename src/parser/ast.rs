use std::rc::Rc;

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Block {
        statements: Vec<Statement>,
    },

    ConditionalExpression {
        condition: Box<Expr>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },

    Expression {
        expression: Box<Expr>,
    },

    FunctionDeclaration {
        name: String,
        params: Rc<Vec<String>>,
        body: Rc<Statement>,
        line: usize,
    },

    Print {
        expression: Box<Expr>,
    },

    Return {
        expression: Option<Box<Expr>>,
        line: usize,
    },

    VarDeclaration {
        name: String,
        initializer: Option<Box<Expr>>,
        line: usize,
    },

    WhileLoop {
        condition: Box<Expr>,
        body: Box<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Number(f64),
    Nil,
    Str(String),
}

#[derive(Debug)]
pub enum Expr {
    Assignment {
        name: String,
        value: Box<Expr>,
        line: usize,
        resolved_depth: Option<usize>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        line: usize,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        line:usize,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: LogicalOperator,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
        line: usize,
    },
    Variable {
        name: String,
        line: usize,
        resolved_depth: Option<usize>,
    },
}

#[derive(Debug)]
pub enum BinaryOperator {
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug)]
pub enum LogicalOperator {
    And,
    Or,
}
