use crate::ast::Expression::Identifier;
use crate::ast::{self, Operator, Statement};
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

    pub fn parse_program(&mut self) -> ast::Program {
        let mut prog = ast::Program {
            statements: Vec::new(),
        };
        while self.current_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                prog.statements.push(stmt);
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
        if self.current_token == Token::Var || self.current_token == Token::Const {
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

            let Token::Identifier(type_name) = self.current_token.clone() else {
                return None;
            };
            self.advance();

            if self.current_token != Token::Equals {
                return None;
            }
            self.advance();

            let Some(initializer) = self.parse_expression(0) else {
                return None;
            };

            if self.current_token == Token::Semicolon {
                self.advance();
            }

            return Some(ast::Statement::Declaration {
                is_mut,
                name,
                type_name,
                initializer,
            });
        } else {
            return None;
        }
    }
    fn parse_statement(&mut self) -> Option<ast::Statement> {
        if self.current_token == Token::Var || self.current_token == Token::Const {
            return self.parse_declaration();
        }
        if let Token::Identifier(name) = &self.current_token {
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
                panic!("Missing Equals!")
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
            Token::Number(val) => ast::Expression::Number(*val),
            Token::Identifier(name) => ast::Expression::Identifier(name.clone()),
            _ => return None,
        };
        self.advance();

        while precedence < Parser::get_precedence(&self.current_token) {
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
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{self, Expression, Operator, Statement},
        lexer::Lexer,
        parser::Parser,
    };

    fn parse(input: &str) -> ast::Program {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    fn num(n: i64) -> ast::Expression {
        ast::Expression::Number(n)
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
        assert_eq!(p.statements.len(), 0);
    }

    #[test]
    fn test_parse_return_statement() {
        let p = parse("return 5;");
        assert_eq!(p.statements.len(), 1);
        assert_eq!(p.statements[0], ret(num(5)));
    }

    #[test]
    fn test_invalid_statement() {
        let p = parse("5;");
        assert_eq!(p.statements.len(), 0);
    }

    #[test]
    fn test_parse_binary_expression() {
        let p = parse("return 5 + 10;");
        assert_eq!(
            p.statements[0],
            ret(bin(num(5), ast::Operator::Add, num(10)))
        );
    }

    #[test]
    fn test_operator_precedence() {
        let p = parse("return 1 + 2 * 3;");
        assert_eq!(
            p.statements[0],
            ret(bin(
                num(1),
                ast::Operator::Add,
                bin(num(2), ast::Operator::Multiply, num(3))
            ))
        );
    }

    #[test]
    fn test_left_associativity() {
        let p = parse("return 1 - 2 - 3;");
        assert_eq!(
            p.statements[0],
            ret(bin(
                bin(num(1), ast::Operator::Subtract, num(2)),
                ast::Operator::Subtract,
                num(3)
            ))
        );
    }

    #[test]
    fn test_parse_const_declaration() {
        let p = parse("const x : i64 = 5;");
        assert_eq!(p.statements.len(), 1);
        assert_eq!(
            p.statements[0],
            Statement::Declaration {
                is_mut: false,
                name: "x".to_string(),
                type_name: "i64".to_string(),
                initializer: ast::Expression::Number(5)
            }
        );
    }

    #[test]
    fn test_parse_identifier() {
        let p = parse("return x;");
        assert_eq!(p.statements.len(), 1);
        assert_eq!(
            p.statements[0],
            Statement::Return(ast::Expression::Identifier("x".to_string()))
        );
    }

    #[test]
    fn test_parse_assignment() {
        let p = parse("x = 10;");
        assert_eq!(p.statements.len(), 1);
        assert_eq!(
            p.statements[0],
            Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Number(10)
            }
        );
    }
}
