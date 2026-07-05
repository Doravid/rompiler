use crate::ast::{self, Operator};
use crate::lexer::{Lexer, Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
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
        self.current_token = self.lexer.next_token();
    }

    fn parse_statement(&mut self) -> Option<ast::Statement> {
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
        if let Token::Number(left_val) = self.current_token {
            let mut left = ast::Expression::Number(left_val);

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
        return None;
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
        ast::{self, Operator},
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
}
