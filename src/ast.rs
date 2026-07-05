#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(i64),
    Identifier(String),
    Binary(Box<Expression>, Operator, Box<Expression>),
}
#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
    Declaration {
        is_mut: bool,
        name: String,
        type_name: String,
        initializer: Expression,
    },
    Assignment {
        name: String,
        value: Expression,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
