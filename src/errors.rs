use crate::token::{Token, TokenType};

#[derive(Default, Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Errors {
    pub had_error: bool,
    pub had_runtime_error: bool,
}

impl Errors {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: usize, loc: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, loc, message);
        self.had_error = true;
    }

    pub fn error_token(&mut self, token: Token, message: &str) {
        if token.r#type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }
}
