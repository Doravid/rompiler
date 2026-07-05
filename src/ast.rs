#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(i64),
    Binary(Box<Expression>, Operator, Box<Expression>),
}
#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
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
