use std::collections::HashMap;

use crate::errors::Errors;
use crate::token::{Object, Token, TokenType};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Scanner {
    pub source: Vec<char>,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub errors: Errors,
    pub keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            line: 1,
            keywords: HashMap::from([
                ("and".to_string(), TokenType::And),
                ("class".to_string(), TokenType::Class),
                ("else".to_string(), TokenType::Else),
                ("false".to_string(), TokenType::False),
                ("for".to_string(), TokenType::For),
                ("fun".to_string(), TokenType::Fun),
                ("if".to_string(), TokenType::If),
                ("nil".to_string(), TokenType::Nil),
                ("or".to_string(), TokenType::Or),
                ("print".to_string(), TokenType::Print),
                ("return".to_string(), TokenType::Return),
                ("super".to_string(), TokenType::Super),
                ("this".to_string(), TokenType::This),
                ("true".to_string(), TokenType::True),
                ("var".to_string(), TokenType::Var),
                ("while".to_string(), TokenType::While),
            ]),
            ..Default::default()
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(TokenType::Eof, None);
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => self.add_relational_token(TokenType::Bang, TokenType::BangEqual),
            '=' => self.add_relational_token(TokenType::Equal, TokenType::EqualEqual),
            '<' => self.add_relational_token(TokenType::Less, TokenType::LessEqual),
            '>' => self.add_relational_token(TokenType::Greater, TokenType::GreaterEqual),
            '/' => {
                if self.r#match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            _ => self.errors.error(self.line, "Unexpected character."),
        }
    }

    fn add_relational_token(&mut self, one_char_token: TokenType, two_char_token: TokenType) {
        let token_to_add = if self.r#match('=') {
            one_char_token
        } else {
            two_char_token
        };
        self.add_token(token_to_add, None);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let str_value: String = self.source[self.start..self.current].iter().collect();

        let value: f64 = str_value.parse().unwrap();

        self.add_token(TokenType::Number, Some(Object::Number(value)));
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn string(&mut self) {
        while self.peek() != '"' && self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.errors.error(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let value: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token(TokenType::String, Some(Object::String(value)));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = self.keywords.get(&text);

        match token_type {
            Some(t_type) => self.add_token(t_type.clone(), None),
            None => self.add_token(TokenType::Identifier, Some(Object::String(text))),
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Object>) {
        self.tokens.push(Token {
            r#type: token_type,
            lexeme: self.source[self.start..self.current].iter().collect(),
            literal,
            line: self.line,
        });
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    macro_rules! test_scanner {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($source.to_string());
                let tokens = scanner.scan_tokens();
                assert_debug_snapshot!(tokens);
            }
        };
    }

    test_scanner!(var_assign, "var x = 10;");
    test_scanner!(fn_call, "fun () { print 10; }");
    test_scanner!(
        while_loop,
        "var x = 10;\n while x <= 20\n{ x += 1;\n print x;\n}"
    );
}
