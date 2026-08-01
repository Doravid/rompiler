#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Identifier(String),
    Binary(Box<Expression>, Operator, Box<Expression>),
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Return(Expression),
    Declaration {
        is_mut: bool,
        name: String,
        type_name: Type,
        initializer: Option<Expression>,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    IndexAssignment {
        name: String,
        index: Expression,
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
    Array(Box<Type>, i64),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}
#[derive(Debug, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}
