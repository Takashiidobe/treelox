use crate::{
    errors::Errors,
    expr::Expr,
    token::{Object, Token, TokenType},
};

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Errors,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        let res = self.expression();
        if self.errors.had_error {
            None
        } else {
            res
        }
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison();

        while self.r#match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Some(Expr::Binary {
                left: Box::new(expr.expect("Could not evaluate left")),
                operator,
                right: Box::new(right.expect("Could not evaluate right")),
            })
        }

        expr
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor();

        while self.r#match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                right: Box::new(right.unwrap()),
            })
        }

        expr
    }

    fn r#match(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().r#type == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term();

        while self.r#match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Some(Expr::Binary {
                left: Box::new(expr.expect("Could not evaluate left")),
                operator,
                right: Box::new(right.expect("Could not evaluate right")),
            })
        }

        expr
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary();

        while self.r#match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Some(Expr::Binary {
                left: Box::new(expr.expect("Could not eval left")),
                operator,
                right: Box::new(right.expect("Could not eval right")),
            })
        }

        expr
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.r#match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            Some(Expr::Unary {
                operator,
                right: Box::new(right.expect("Could not evaluate right")),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.r#match(&[TokenType::False]) {
            return Some(Expr::Literal {
                value: Object::Bool(false),
            });
        }

        if self.r#match(&[TokenType::True]) {
            return Some(Expr::Literal {
                value: Object::Bool(true),
            });
        }

        if self.r#match(&[TokenType::Nil]) {
            return Some(Expr::Literal { value: Object::Nil });
        }

        if self.r#match(&[TokenType::Number, TokenType::String]) {
            return Some(Expr::Literal {
                value: self.previous().literal.unwrap(),
            });
        }

        if self.r#match(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(&TokenType::RightParen, "Expect ')' after expression.");

            return Some(Expr::Grouping {
                expr: Box::new(expr.expect("Could not evaluate expr")),
            });
        }

        self.errors.error_token(self.peek(), "Expect expression.");
        None
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Option<Token> {
        if self.check(token_type) {
            return Some(self.advance());
        }

        self.errors.error_token(self.peek(), message);
        None
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().r#type == TokenType::Semicolon {
                return;
            }

            match self.peek().r#type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use insta::assert_debug_snapshot;

    macro_rules! test_parser {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($source.to_string());
                let tokens = scanner.scan_tokens();
                let mut parser = Parser::new(tokens);
                assert_debug_snapshot!(parser.parse());
            }
        };
    }

    test_parser!(precedence_math, "15 - 3 * 4");
    test_parser!(grouping, "(5 - 3) * 4");
    test_parser!(parse_true, "true");
    test_parser!(parse_false, "false");
    test_parser!(parse_nil, "nil");
}
