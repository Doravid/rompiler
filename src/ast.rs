#[derive(Debug, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Identifier(String),
    Binary(Box<Expression>, Operator, Box<Expression>),
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
}
#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
    Declaration {
        is_mut: bool,
        name: String,
        type_name: Type,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Pointer(Box<Type>),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
