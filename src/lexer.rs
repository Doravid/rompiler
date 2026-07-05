#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Eof,
    Semicolon,
    Return,
    Number(i64),
    Plus,
    Const,
    Var,
    Colon,
    Equals,
    Identifier(String),
    Minus,
    Asterisk,
    Slash,
    Illegal(String),
}
pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }

        let Some(ch) = self.input.next() else {
            return Token::Eof;
        };
        match ch {
            ';' => Token::Semicolon,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            ':' => Token::Colon,
            '=' => Token::Equals,
            '0'..='9' => self.lex_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(ch),
            _ => Token::Illegal(ch.to_string()),
        }
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut my_string = String::from(first);

        while let Some(&c) = self.input.peek() {
            if c.is_ascii_digit() {
                my_string.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        return Token::Number(my_string.parse().unwrap());
    }

    fn lex_identifier(&mut self, first: char) -> Token {
        let mut my_string = String::from(first);

        while let Some(&c) = self.input.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                my_string.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        match my_string.as_str() {
            "return" => return Token::Return,
            "const" => return Token::Const,
            "var" => return Token::Var,
            _ => return Token::Identifier(my_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input_returns_eof() {
        let mut lexer: Lexer<'_> = Lexer::new("");
        let my_token: Token = lexer.next_token();

        assert_eq!(my_token, Token::Eof);
    }
    #[test]
    fn test_semicolon() {
        let mut lexer = Lexer::new(" ; ");
        let my_token: Token = lexer.next_token();

        assert_eq!(my_token, Token::Semicolon);
        let my_token2: Token = lexer.next_token();
        assert_eq!(my_token2, Token::Eof);
    }
    #[test]
    fn test_single_digit_number() {
        let mut lexer = Lexer::new(" 5 ");
        let my_token: Token = lexer.next_token();

        assert_eq!(my_token, Token::Number(5));
    }
    #[test]
    fn test_multi_digit_number() {
        let mut lexer = Lexer::new(" 505 ");
        let my_token: Token = lexer.next_token();

        assert_eq!(my_token, Token::Number(505));
    }
    #[test]

    fn test_return() {
        let mut lexer = Lexer::new(" return 505 ");
        let my_token: Token = lexer.next_token();

        assert_eq!(my_token, Token::Return);
    }

    #[test]
    fn test_full_statement() {
        let mut lexer = Lexer::new(" return 505; ");
        let return_token: Token = lexer.next_token();
        let number_token: Token = lexer.next_token();
        let semicolon_token: Token = lexer.next_token();
        let eof_token: Token = lexer.next_token();

        assert_eq!(return_token, Token::Return);
        assert_eq!(number_token, Token::Number(505));
        assert_eq!(semicolon_token, Token::Semicolon);
        assert_eq!(eof_token, Token::Eof);
    }

    #[test]
    fn test_math_operators() {
        let mut lexer = Lexer::new("    + - * / ");
        let plus_token: Token = lexer.next_token();
        let minus_token: Token = lexer.next_token();
        let asterisk_token: Token = lexer.next_token();
        let slash_token: Token = lexer.next_token();

        assert_eq!(plus_token, Token::Plus);
        assert_eq!(minus_token, Token::Minus);
        assert_eq!(asterisk_token, Token::Asterisk);
        assert_eq!(slash_token, Token::Slash);
    }

    #[test]
    fn test_variable_tokens() {
        let mut lexer = Lexer::new("const x : i64 = 5;");
        let token1: Token = lexer.next_token();
        let token2: Token = lexer.next_token();
        let token3: Token = lexer.next_token();
        let token4: Token = lexer.next_token();
        let token5: Token = lexer.next_token();
        let token6: Token = lexer.next_token();
        let token7: Token = lexer.next_token();
        let token8: Token = lexer.next_token();

        assert_eq!(token1, Token::Const);
        assert_eq!(token2, Token::Identifier(String::from("x")));
        assert_eq!(token3, Token::Colon);
        assert_eq!(token4, Token::Identifier(String::from("i64")));
        assert_eq!(token5, Token::Equals);
        assert_eq!(token6, Token::Number(5));
        assert_eq!(token7, Token::Semicolon);
        assert_eq!(token8, Token::Eof);
    }
}
