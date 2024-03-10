use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Environment,
    error::Error,
    expr::{expr, Expr},
    function::Function,
    stmt::{stmt, Stmt},
    token::{Object, Token, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let clock: Object = Object::Callable(Function::Native {
            arity: 0,
            body: Box::new(|_: &[Object]| {
                Object::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        });
        globals.borrow_mut().define(
            &Token {
                r#type: TokenType::Fun,
                lexeme: "clock".to_string(),
                literal: None,
                line: 0,
            },
            clock,
        );
        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::clone(&globals),
            locals: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
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

    pub(crate) fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.clone(), depth);
    }

    fn look_up_variable(&self, name: &Token) -> Result<Object, Error> {
        if let Some(distance) = self.locals.get(name) {
            self.environment.borrow().get_at(*distance, name)
        } else {
            self.globals.borrow().get(name)
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        let previous = self.environment.clone();
        let steps = || -> Result<(), Error> {
            self.environment = environment;
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

    fn visit_variable_expr(&mut self, name: &Token) -> Result<Object, Error> {
        self.look_up_variable(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Object, Error> {
        let value = self.evaluate(value)?;
        if let Some(distance) = self.locals.get(name) {
            self.environment
                .borrow_mut()
                .assign_at(*distance, name, value.clone())?;
        } else {
            self.environment.borrow_mut().assign(name, value.clone())?;
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

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
    ) -> Result<Object, Error> {
        let callee = self.evaluate(callee)?;

        let mut args = vec![];

        for argument in arguments {
            args.push(self.evaluate(argument)?);
        }

        if let Object::Callable(function) = callee {
            let arg_count = args.len();
            if arg_count != function.arity() {
                Err(Error::Runtime {
                    token: paren.clone(),
                    message: format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        arg_count
                    ),
                })
            } else {
                function.call(self, &args)
            }
        } else {
            Err(Error::Runtime {
                token: paren.clone(),
                message: "Can only call functions and classes.".to_string(),
            })
        }
    }
}

impl stmt::Visitor<()> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        self.execute_block(
            statements,
            Rc::new(RefCell::new(Environment::from(&self.environment))),
        )?;
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

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &[Stmt],
    ) -> Result<(), Error> {
        let function = Function::User {
            name: Box::new(name.clone()),
            params: params.to_vec(),
            body: body.to_vec(),
            closure: Rc::clone(&self.environment),
        };
        self.environment
            .borrow_mut()
            .define(name, Object::Callable(function));
        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: &Token, value: &Option<Expr>) -> Result<(), Error> {
        let return_value = if let Some(val) = value {
            self.evaluate(val)?
        } else {
            Object::Nil
        };

        Err(Error::Return {
            value: return_value,
        })
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
