use std::{
    fs::read_to_string,
    io::{self, Write},
    process,
};

use crate::{
    expr::{Expr, Visitor},
    parser::Parser,
    scanner::Scanner,
    token::{Object, Token, TokenType},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Interpreter {
    args: Vec<String>,
    had_error: bool,
    had_runtime_error: bool,
}

impl Interpreter {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            ..Default::default()
        }
    }

    pub fn execute(&mut self) {
        match self.args.len() {
            1 => self.run_prompt(),
            2 => self.run_file(self.args[1].to_string()).unwrap(),
            _ => {
                eprintln!("Usage: treelox [path]");
                process::exit(64);
            }
        }
    }

    fn run_prompt(&mut self) {
        loop {
            let mut line = String::new();

            print!("> ");
            io::stdout().flush().unwrap();

            if let Err(e) = io::stdin().read_line(&mut line) {
                eprintln!("{}", e);
                break;
            }

            let _ = self.run(line.clone());
        }
    }

    fn run_file(&mut self, path: String) -> Result<(), io::Error> {
        let source = read_to_string(path)?;
        let run_result = self.run(source);
        if let Err(()) = run_result {
            if self.had_error {
                process::exit(65);
            }
            if self.had_runtime_error {
                process::exit(70);
            }
        }
        Ok(())
    }

    fn run(&mut self, source: String) -> Result<(), ()> {
        let run_result = self.run_result(source);
        match run_result {
            Ok(obj) => {
                println!("{}", obj);
                Ok(())
            }
            Err((compile_time_err, runtime_err)) => {
                if compile_time_err {
                    self.had_error = compile_time_err;
                }
                if runtime_err {
                    self.had_runtime_error = runtime_err;
                }
                Err(())
            }
        }
    }

    fn run_result(&mut self, source: String) -> Result<Object, (bool, bool)> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse();

        match expr {
            Some(parsed_expr) => {
                let value = self.evaluate(&parsed_expr);
                match value {
                    Some(res) => Ok(res),
                    None => {
                        self.had_runtime_error = true;
                        Err((false, true))
                    }
                }
            }
            None => {
                self.had_error = true;
                Err((true, false))
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Option<Object> {
        expr.accept(self)
    }

    fn runtime_error(&self, left: &Object, operator: &Token, right: &Object) -> Option<Object> {
        match operator.r#type {
            TokenType::Minus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                eprintln!(
                    "Operands must be numbers. Was: {} {} {}",
                    left, operator, right
                );
            }
            TokenType::Plus => {
                eprintln!(
                    "Operands must be two numbers or two strings. Was: {} {} {}",
                    left, operator, right
                );
            }
            _ => {
                eprintln!(
                    "Invalid expression error. Was: {} {} {}",
                    left, operator, right
                );
            }
        }
        None
    }
}

impl Visitor<Option<Object>> for Interpreter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Option<Object> {
        let left = self
            .evaluate(left)
            .unwrap_or_else(|| panic!("Could not evaluate left expr: {:?}", left));
        let right = self
            .evaluate(right)
            .unwrap_or_else(|| panic!("Could not evaluate right expr: {:?}", right));

        match (&left, &operator.r#type, &right) {
            (Object::Number(left_num), TokenType::Minus, Object::Number(right_num)) => {
                Some(Object::Number(left_num - right_num))
            }
            (Object::Number(left_num), TokenType::Slash, Object::Number(0.0)) => {
                eprintln!("Zero division error. Tried to divide {} by 0.", left_num);
                None
            }
            (Object::Number(left_num), TokenType::Slash, Object::Number(right_num)) => {
                Some(Object::Number(left_num / right_num))
            }
            (Object::Number(left_num), TokenType::Star, Object::Number(right_num)) => {
                Some(Object::Number(left_num * right_num))
            }
            (Object::Number(left_num), TokenType::Plus, Object::Number(right_num)) => {
                Some(Object::Number(left_num + right_num))
            }
            (Object::String(left_str), TokenType::Plus, Object::String(right_str)) => {
                let mut res = left_str.clone();
                res.push_str(right_str);
                Some(Object::String(res))
            }
            (Object::Number(left_num), TokenType::Greater, Object::Number(right_num)) => {
                Some(Object::Bool(left_num > right_num))
            }
            (Object::Number(left_num), TokenType::GreaterEqual, Object::Number(right_num)) => {
                Some(Object::Bool(left_num >= right_num))
            }
            (Object::Number(left_num), TokenType::Less, Object::Number(right_num)) => {
                Some(Object::Bool(left_num < right_num))
            }
            (Object::Number(left_num), TokenType::LessEqual, Object::Number(right_num)) => {
                Some(Object::Bool(left_num <= right_num))
            }
            (_, TokenType::BangEqual, _) => Some(Object::Bool(left != right)),
            (_, TokenType::EqualEqual, _) => Some(Object::Bool(left == right)),
            _ => self.runtime_error(&left, operator, &right),
        }
    }

    fn visit_grouping_expr(&self, expr: &Expr) -> Option<Object> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, value: &Object) -> Option<Object> {
        Some(value.clone())
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Option<Object> {
        let right = self.evaluate(right);

        match (operator.r#type.clone(), right.unwrap()) {
            (TokenType::Minus, Object::Number(num)) => Some(Object::Number(-num)),
            (TokenType::Bang, obj) => Some(Object::Bool(!obj.is_truthy())),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    macro_rules! test_repl {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut interpreter = Interpreter::new(vec!["treelox".to_string()]);
                let mut results = vec![];
                for src in $source {
                    results.push(interpreter.run_result(src.to_string()));
                }
                assert_debug_snapshot!(results);
            }
        };
    }

    test_repl!(grouping_math, &["(40 - 30) * 20", "60 - 20 * 40"]);
    test_repl!(error, &["(40 "]);
}
