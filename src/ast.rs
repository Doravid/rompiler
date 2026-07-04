#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(i64),
}
#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}
#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
