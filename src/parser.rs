use crate::ast::{self, Statement};
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

        if let Token::Number(val) = self.current_token {
            self.advance();

            if self.current_token == Token::Semicolon {
                self.advance();
            }

            return Some(ast::Statement::Return(ast::Expression::Number(val)));
        }

        return None;
    }
}

#[test]
fn test_empty_program() {
    let lexer: Lexer<'_> = Lexer::new("");
    let mut parser = Parser::new(lexer);
    let p: ast::Program = parser.parse_program();

    assert_eq!(p.statements.len(), 0);
}

#[test]
fn test_parse_return_statement() {
    let mut lexer: Lexer<'_> = Lexer::new("return 5;");
    let mut parser = Parser::new(lexer);
    let p: ast::Program = parser.parse_program();

    assert_eq!(p.statements.len(), 1);
    assert_eq!(
        p.statements[0],
        ast::Statement::Return(ast::Expression::Number(5))
    );
}
#[test]
fn test_invalid_statement() {
    let mut lexer: Lexer<'_> = Lexer::new("5;");
    let mut parser = Parser::new(lexer);
    let p: ast::Program = parser.parse_program();

    assert_eq!(p.statements.len(), 0);
}
