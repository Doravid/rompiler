use crate::ast::{self, Expression, Function, Operator, Statement, Type};
use crate::lexer::{Lexer, Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }
    fn parse_type_from_string(&mut self, name: &str) -> Option<ast::Type> {
        match name {
            "i8" => Some(ast::Type::I8),
            "i16" => Some(ast::Type::I16),
            "i32" => Some(ast::Type::I32),
            "i64" => Some(ast::Type::I64),
            "u8" => Some(ast::Type::U8),
            "u16" => Some(ast::Type::U16),
            "u32" => Some(ast::Type::U32),
            "u64" => Some(ast::Type::U64),
            "f32" => Some(ast::Type::F32),
            "f64" => Some(ast::Type::F64),
            _ => None,
        }
    }

    fn parse_function(&mut self) -> Option<Function> {
        if self.current_token != Token::Function {
            return None;
        }

        self.advance();
        let Token::Identifier(func_name) = self.current_token.clone() else {
            return None;
        };
        self.advance();

        if self.current_token != Token::LeftParenthesis {
            return None;
        }
        self.advance();

        let mut parameters: Vec<(String, Type)> = vec![];
        while self.current_token != Token::RightParenthesis {
            let Token::Identifier(var_name) = self.current_token.clone() else {
                return None;
            };
            self.advance();
            if self.current_token != Token::Colon {
                return None;
            }
            self.advance();
            let var_type = self.parse_type().unwrap();
            parameters.push((var_name, var_type));
            if self.current_token != Token::Comma && self.current_token != Token::RightParenthesis {
                return None;
            }
            if self.current_token == Token::Comma {
                self.advance();
            }
        }
        self.advance();
        if self.current_token != Token::Colon {
            return None;
        }
        self.advance();
        let func_type = self.parse_type().unwrap();

        if self.current_token != Token::LeftCurlyBrace {
            return None;
        }
        self.advance();
        let mut statements: Vec<ast::Statement> = vec![];
        while self.current_token != Token::RightCurlyBrace {
            statements.push(self.parse_statement().unwrap());
        }

        return Some(Function {
            name: func_name,
            parameters,
            return_type: func_type,
            body: statements,
        });
    }

    fn parse_type(&mut self) -> Option<ast::Type> {
        let is_pointer = self.current_token == Token::Asterisk;
        if is_pointer {
            self.advance();
        }
        let is_array = self.current_token == Token::LeftBracket;

        if is_array {
            self.advance();
            let Token::Integer(array_size) = self.current_token.clone() else {
                return None;
            };
            self.advance();
            assert!(self.current_token == Token::RightBracket);
            self.advance();
            let Token::Identifier(type_name) = self.current_token.clone() else {
                return None;
            };
            self.advance();
            let typ = self.parse_type_from_string(&type_name).unwrap();
            if is_pointer {
                return Some(Type::Pointer(Box::new(Type::Array(
                    Box::new(typ),
                    array_size,
                ))));
            } else {
                return Some(Type::Array(Box::new(typ), array_size));
            }
        }

        let Token::Identifier(type_name) = self.current_token.clone() else {
            return None;
        };
        self.advance();

        let typ = self.parse_type_from_string(&type_name).unwrap();
        if is_pointer {
            return Some(Type::Pointer(Box::new(typ)));
        } else {
            return Some(typ);
        }
    }

    pub fn parse_program(&mut self) -> ast::Program {
        let mut prog = ast::Program {
            functions: Vec::new(),
        };
        while self.current_token != Token::Eof {
            if let Some(func) = self.parse_function() {
                prog.functions.push(func);
            } else {
                self.advance();
            }
        }
        return prog;
    }

    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
    fn parse_declaration(&mut self) -> Option<ast::Statement> {
        let is_mut = self.current_token == Token::Var;

        self.advance();
        let Token::Identifier(name) = self.current_token.clone() else {
            return None;
        };
        self.advance();

        if self.current_token != Token::Colon {
            return None;
        }

        self.advance();
        let Some(value_type) = self.parse_type() else {
            return None;
        };

        if self.current_token == Token::Semicolon {
            self.advance();
            return Some(ast::Statement::Declaration {
                is_mut,
                name,
                type_name: value_type,
                initializer: None,
            });
        }

        if self.current_token != Token::Equals {
            return None;
        }
        self.advance();

        let Some(initializer) = self.parse_expression(0) else {
            return None;
        };

        if self.current_token != Token::Semicolon {
            return None;
        }
        self.advance();

        return Some(ast::Statement::Declaration {
            is_mut,
            name,
            type_name: value_type,
            initializer: Some(initializer),
        });
    }
    fn parse_statement(&mut self) -> Option<ast::Statement> {
        if self.current_token == Token::Var || self.current_token == Token::Const {
            return self.parse_declaration();
        }

        if let Token::Identifier(name) = &self.current_token {
            let var_name = name.clone();

            if self.peek_token == Token::LeftBracket {
                self.advance();
                self.advance();
                let Some(index) = self.parse_expression(0) else {
                    panic!(
                        "Failed to parse expression for array index in setting a value in the array."
                    );
                };
                if self.current_token != Token::RightBracket {
                    panic!("No right bracket on array index");
                }
                self.advance();
                if self.current_token != Token::Equals {
                    panic!("No equals sign in array setting statement");
                }
                self.advance();
                let Some(value) = self.parse_expression(0) else {
                    panic!("Invalid expression. Cannot set array value.");
                };
                if self.current_token != Token::Semicolon {
                    panic!("Array statement not ended with a semicolon!");
                }
                self.advance();
                return Some(Statement::IndexAssignment {
                    name: var_name,
                    index,
                    value,
                });
            }

            if self.peek_token == Token::Equals {
                let var_name = name.clone();
                self.advance();
                self.advance();

                if let Some(expr) = self.parse_expression(0) {
                    if self.current_token == Token::Semicolon {
                        self.advance();
                    } else {
                        panic!("Missing Semicolon!")
                    }
                    return Some(ast::Statement::Assignment {
                        name: var_name,
                        value: expr,
                    });
                }
            } else {
                return None;
            }
        }

        if self.current_token == Token::Return {
            self.advance();
        } else {
            return None;
        }
        if let Some(expr) = self.parse_expression(0) {
            if self.current_token == Token::Semicolon {
                self.advance();
            }
            return Some(ast::Statement::Return(expr));
        }

        return None;
    }

    fn parse_expression(&mut self, precedence: u8) -> Option<ast::Expression> {
        let mut left = match &self.current_token {
            Token::Integer(val) => {
                let expr = ast::Expression::Integer(*val);
                self.advance();
                expr
            }
            Token::Float(val) => {
                let expr = ast::Expression::Float(*val);
                self.advance();
                expr
            }
            Token::Identifier(name) => {
                let expr = ast::Expression::Identifier(name.clone());
                self.advance();
                expr
            }
            Token::Ampersand => {
                self.advance();
                ast::Expression::AddressOf(Box::new(self.parse_expression(3).unwrap()))
            }
            Token::Asterisk => {
                self.advance();
                ast::Expression::Dereference(Box::new(self.parse_expression(3).unwrap()))
            }
            _ => return None,
        };

        while precedence < Parser::get_precedence(&self.current_token) {
            if self.current_token == Token::LeftBracket {
                self.advance();
                let Some(expr) = self.parse_expression(0) else {
                    return None;
                };
                if self.current_token != Token::RightBracket {
                    return None;
                }
                self.advance();
                return Some(ast::Expression::Index(Box::new(left), Box::new(expr)));
            }

            if self.current_token == Token::LeftParenthesis {
                let Expression::Identifier(function_name) = left else {
                    return None;
                };
                self.advance();
                let mut x: Vec<Expression> = vec![];
                while self.current_token != Token::RightParenthesis {
                    x.push(self.parse_expression(0).unwrap());
                    if self.current_token == Token::Comma {
                        self.advance();
                    } else if self.current_token != Token::RightParenthesis {
                        return None;
                    }
                }
                self.advance();
                return Some(Expression::Call(function_name, x));
            }

            let operator: Operator = match self.current_token {
                Token::Asterisk => ast::Operator::Multiply,
                Token::Plus => ast::Operator::Add,
                Token::Minus => ast::Operator::Subtract,
                Token::Slash => ast::Operator::Divide,

                _ => return Some(left),
            };
            let cur_precedence = Parser::get_precedence(&self.current_token);
            self.advance();

            let right = self.parse_expression(cur_precedence);
            left = ast::Expression::Binary(Box::new(left), operator, Box::new(right.unwrap()));
        }
        return Some(left);
    }
    fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::Plus | Token::Minus => 1,
            Token::Asterisk | Token::Slash => 2,
            Token::LeftBracket | Token::LeftParenthesis => 3,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{self, Expression, Function, Statement, Type},
        lexer::Lexer,
        parser::Parser,
    };

    fn parse(input: &str) -> ast::Program {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    fn num(n: i64) -> ast::Expression {
        ast::Expression::Integer(n)
    }

    fn bin(left: ast::Expression, op: ast::Operator, right: ast::Expression) -> ast::Expression {
        ast::Expression::Binary(Box::new(left), op, Box::new(right))
    }

    fn ret(expr: ast::Expression) -> ast::Statement {
        ast::Statement::Return(expr)
    }

    #[test]
    fn test_empty_program() {
        let p = parse("");
        assert_eq!(p.functions.len(), 0);
    }

    #[test]
    fn test_parse_return_statement() {
        let p = parse("func main() : i32 { return 5; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(p.functions[0].body[0], ret(num(5)));
    }

    #[test]
    fn test_invalid_statement() {
        let p = parse("5;");
        assert_eq!(p.functions.len(), 0);
    }

    #[test]
    fn test_parse_binary_expression() {
        let p = parse("func main() : i32 { return 5 + 10; }");
        assert_eq!(
            p.functions[0].body[0],
            ret(bin(num(5), ast::Operator::Add, num(10)))
        );
    }

    #[test]
    fn test_operator_precedence() {
        let p = parse("func main() : i32 { return 1 + 2 * 3; }");
        assert_eq!(
            p.functions[0].body[0],
            ret(bin(
                num(1),
                ast::Operator::Add,
                bin(num(2), ast::Operator::Multiply, num(3))
            ))
        );
    }

    #[test]
    fn test_left_associativity() {
        let p = parse("func main() : i32 { return 1 - 2 - 3; }");
        assert_eq!(
            p.functions[0].body[0],
            ret(bin(
                bin(num(1), ast::Operator::Subtract, num(2)),
                ast::Operator::Subtract,
                num(3)
            ))
        );
    }

    #[test]
    fn test_parse_const_declaration() {
        let p = parse("func main() : i32 { const x : i64 = 5; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "x".to_string(),
                type_name: Type::I64,
                initializer: Some(ast::Expression::Integer(5))
            }
        );
    }

    #[test]
    fn test_parse_identifier() {
        let p = parse("func main() : i32 { return x; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Return(ast::Expression::Identifier("x".to_string()))
        );
    }

    #[test]
    fn test_parse_assignment() {
        let p = parse("func main() : i32 { x = 10; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Integer(10)
            }
        );
    }

    #[test]
    fn test_parse_float_declaration() {
        let p = parse("func main() : i32 { const pi : f64 = 3.14; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "pi".to_string(),
                type_name: Type::F64,
                initializer: Some(ast::Expression::Float(3.14))
            }
        );
    }

    #[test]
    fn test_parse_pointer_declaration() {
        let p = parse("func main() : i32 { const p : *i64 = &x; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "p".to_string(),
                type_name: ast::Type::Pointer(Box::new(ast::Type::I64)),
                initializer: Some(ast::Expression::AddressOf(Box::new(
                    ast::Expression::Identifier("x".to_string())
                )))
            }
        );
    }

    #[test]
    fn test_parse_dereference() {
        let p = parse("func main() : i32 { return *p; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Return(ast::Expression::Dereference(Box::new(
                ast::Expression::Identifier("p".to_string())
            )))
        );
    }

    #[test]
    fn test_parse_uninitialized() {
        let p = parse("func main() : i32 { const x: i32; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "x".to_string(),
                type_name: ast::Type::I32,
                initializer: None
            }
        );
    }

    #[test]
    fn test_parse_array() {
        let p = parse("func main() : i32 { const x: [512]i32; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "x".to_string(),
                type_name: ast::Type::Array(Box::new(Type::I32), 512),
                initializer: None
            }
        );
    }

    #[test]
    fn test_parse_pointer_to_array() {
        let p = parse("func main() : i32 { const x: *[512]i32; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "x".to_string(),
                type_name: Type::Pointer(Box::new(Type::Array(Box::new(Type::I32), 512))),
                initializer: None
            }
        );
    }

    #[test]
    fn test_parse_array_assignment() {
        let p = parse("func main() : i32 { x[12] = 21; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::IndexAssignment {
                name: "x".to_string(),
                index: Expression::Integer(12),
                value: Expression::Integer(21)
            }
        );
    }

    #[test]
    fn test_parse_array_read() {
        let p = parse("func main() : i32 { const y: u32 = x[12]; }");
        assert_eq!(p.functions[0].body.len(), 1);
        assert_eq!(
            p.functions[0].body[0],
            Statement::Declaration {
                is_mut: false,
                name: "y".to_string(),
                type_name: Type::U32,
                initializer: Some(Expression::Index(
                    Box::new(Expression::Identifier("x".to_string())),
                    Box::new(Expression::Integer(12))
                ))
            }
        );
    }

    #[test]
    fn test_parse_function_decl() {
        let p = parse("func main(x: i32) : i32 { return x; }");
        assert_eq!(p.functions.len(), 1);
        assert_eq!(
            p.functions[0],
            Function {
                name: "main".to_string(),
                parameters: vec![("x".to_string(), Type::I32)],
                return_type: Type::I32,
                body: vec![Statement::Return(Expression::Identifier("x".to_string()))]
            }
        );
    }

    #[test]
    fn test_parse_function_call() {
        let p = parse("func main() : i32 { return add(5, 10); }");
        assert_eq!(p.functions.len(), 1);
        assert_eq!(
            p.functions[0],
            Function {
                name: "main".to_string(),
                parameters: vec![],
                return_type: Type::I32,
                body: vec![Statement::Return(Expression::Call(
                    "add".to_string(),
                    vec![Expression::Integer(5), Expression::Integer(10)]
                ))]
            }
        );
    }
}
