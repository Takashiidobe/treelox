use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    error::Error,
    expr::{expr, Expr},
    stmt::{stmt, Stmt},
    token::{Object, Token, TokenType},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    pub fn interpret_expressions(&mut self, expressions: &Vec<Expr>) -> Result<(), Error> {
        for expression in expressions {
            println!("{}", self.evaluate(expression)?);
        }
        Ok(())
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Object, Error> {
        expression.accept(self)
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), Error> {
        statement.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), Error> {
        let previous = self.environment.clone();
        let steps = || -> Result<(), Error> {
            self.environment = Rc::new(RefCell::new(environment));
            for statement in statements {
                self.execute(statement)?
            }
            Ok(())
        };
        let result = steps();
        self.environment = previous;
        result
    }

    fn runtime_error(
        &self,
        left: &Object,
        operator: &Token,
        right: &Object,
    ) -> Result<Object, Error> {
        let message = match operator.r#type {
            TokenType::Minus
            | TokenType::Slash
            | TokenType::Star
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                format!(
                    "Operands must be numbers. Was: {} {} {}",
                    left, operator, right
                )
            }
            TokenType::Plus => {
                format!(
                    "Operands must be two numbers or two strings. Was: {} {} {}",
                    left, operator, right
                )
            }
            _ => {
                format!(
                    "Invalid expression error. Was: {} {} {}",
                    left, operator, right
                )
            }
        };
        Err(Error::Runtime {
            token: operator.clone(),
            message,
        })
    }
}

impl expr::Visitor<Object> for Interpreter {
    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, Error> {
        let left = self
            .evaluate(left)
            .unwrap_or_else(|_| panic!("Could not evaluate left expr: {:?}", left));
        let right = self
            .evaluate(right)
            .unwrap_or_else(|_| panic!("Could not evaluate right expr: {:?}", right));

        match (&left, &operator.r#type, &right) {
            (Object::Number(left_num), TokenType::Minus, Object::Number(right_num)) => {
                Ok(Object::Number(left_num - right_num))
            }
            (Object::Number(left_num), TokenType::Slash, Object::Number(0.0)) => {
                Err(Error::Runtime {
                    token: operator.clone(),
                    message: format!("Zero division error. Tried to divide {} by 0.", left_num),
                })
            }
            (Object::Number(left_num), TokenType::Slash, Object::Number(right_num)) => {
                Ok(Object::Number(left_num / right_num))
            }
            (Object::Number(left_num), TokenType::Star, Object::Number(right_num)) => {
                Ok(Object::Number(left_num * right_num))
            }
            (Object::Number(left_num), TokenType::Plus, Object::Number(right_num)) => {
                Ok(Object::Number(left_num + right_num))
            }
            (Object::String(left_str), TokenType::Plus, Object::String(right_str)) => {
                Ok(Object::String(left_str.to_owned() + right_str))
            }
            (Object::Number(left_num), TokenType::Greater, Object::Number(right_num)) => {
                Ok(Object::Bool(left_num > right_num))
            }
            (Object::Number(left_num), TokenType::GreaterEqual, Object::Number(right_num)) => {
                Ok(Object::Bool(left_num >= right_num))
            }
            (Object::Number(left_num), TokenType::Less, Object::Number(right_num)) => {
                Ok(Object::Bool(left_num < right_num))
            }
            (Object::Number(left_num), TokenType::LessEqual, Object::Number(right_num)) => {
                Ok(Object::Bool(left_num <= right_num))
            }
            (_, TokenType::BangEqual, _) => Ok(Object::Bool(left != right)),
            (_, TokenType::EqualEqual, _) => Ok(Object::Bool(left == right)),
            _ => self.runtime_error(&left, operator, &right),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<Object, Error> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, value: &Object) -> Result<Object, Error> {
        Ok(value.clone())
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Object, Error> {
        let right = self.evaluate(right)?;

        match (operator.r#type.clone(), right.clone()) {
            (TokenType::Minus, Object::Number(num)) => Ok(Object::Number(-num)),
            (TokenType::Bang, obj) => Ok(Object::Bool(!obj.is_truthy())),
            _ => Err(Error::Runtime {
                token: operator.clone(),
                message: format!("invalid unary expr: {:?}{:?}", operator, right),
            }),
        }
    }

    fn visit_variable_expr(&self, name: &Token) -> Result<Object, Error> {
        self.environment.borrow().get(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Object, Error> {
        let value = self.evaluate(value)?;
        if let Some(Object::String(_)) = name.literal.clone() {
            self.environment.borrow_mut().assign(name, value.clone())?
        }
        Ok(value)
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, Error> {
        let left = self.evaluate(left)?;
        if operator.r#type == TokenType::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else if !left.is_truthy() {
            return Ok(left);
        }
        self.evaluate(right)
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        self.execute_block(statements, Environment::from(&self.environment))?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        match self.evaluate(expression) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<(), Error> {
        match self.evaluate(expression) {
            Ok(value) => {
                println!("{}", value);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<(), Error> {
        let value: Object = initializer
            .as_ref()
            .map(|i| self.evaluate(i))
            .unwrap_or(Ok(Object::Nil))?;

        self.environment.borrow_mut().define(name, value);
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> Result<(), Error> {
        if self.evaluate(condition).is_ok_and(|obj| obj.is_truthy()) {
            self.execute(then_branch)
        } else if else_branch.is_some() {
            self.execute(else_branch.as_ref().unwrap())
        } else {
            Ok(())
        }
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Error> {
        while self.evaluate(condition).is_ok_and(|obj| obj.is_truthy()) {
            self.execute(body)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    use insta::assert_debug_snapshot;

    macro_rules! test_source_file {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($source.to_string());
                let tokens = scanner.scan_tokens();

                let mut parser = Parser::new(tokens);
                let statements = parser.parse();
                match statements {
                    Ok(statements) => {
                        let mut interpreter = Interpreter::new();

                        let mut results = vec![];

                        for statement in statements {
                            results.push(interpreter.execute(&statement));
                        }
                        assert_debug_snapshot!(results);
                    }
                    Err(_) => assert_debug_snapshot!(statements),
                }
            }
        };
    }

    test_source_file!(grouping_math, "var x = (40 - 30) * 20;");
    test_source_file!(error, "(40");

    macro_rules! test_repl {
        ($name:ident, $source:expr) => {
            #[test]
            fn $name() {
                let mut interpreter = Interpreter::new();
                let mut results = vec![];
                for line in $source {
                    let mut scanner = Scanner::new(line.to_string());
                    let tokens = scanner.scan_tokens();

                    let mut parser = Parser::new(tokens);
                    let expressions = parser.parse_exprs();
                    match expressions {
                        Ok(exprs) => {
                            for expr in exprs {
                                dbg!(&interpreter);
                                results.push(interpreter.evaluate(&expr));
                            }

                            assert_debug_snapshot!(results);
                        }
                        Err(_) => assert_debug_snapshot!(expressions),
                    }
                }
            }
        };
    }

    test_repl!(var_assign, &["var x = (40 - 30) * 20;", "print x;"]);
    test_repl!(repl_err, &["(40", "var x = 10;", "print x;"]);
}
