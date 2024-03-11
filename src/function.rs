use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Object;
use crate::token::Token;
use crate::token::TokenType;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&[Object]) -> Object>,
    },

    User {
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    },
}

impl Function {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object, Error> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
            Function::User {
                params,
                body,
                closure,
                is_initializer,
                ..
            } => {
                let environment = Rc::new(RefCell::new(Environment::from(closure)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment.borrow_mut().define(param, argument.clone());
                }
                match interpreter.execute_block(body, environment) {
                    Err(Error::Return { value }) => {
                        if *is_initializer {
                            Ok(closure
                                .borrow()
                                .get_at(
                                    0,
                                    &Token {
                                        r#type: TokenType::This,
                                        lexeme: "this".to_string(),
                                        literal: Some(Object::Identifier("this".to_string())),
                                        line: 0,
                                    },
                                )
                                .expect("Initializer should return 'this'."))
                        } else {
                            Ok(value)
                        }
                    }
                    Err(other) => Err(other),
                    Ok(..) => {
                        if *is_initializer {
                            Ok(closure
                                .borrow()
                                .get_at(
                                    0,
                                    &Token {
                                        r#type: TokenType::This,
                                        lexeme: "this".to_string(),
                                        literal: Some(Object::Identifier("this".to_string())),
                                        line: 0,
                                    },
                                )
                                .expect("Initializer should return 'this'."))
                        } else {
                            Ok(Object::Nil)
                        }
                    }
                }
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::User { params, .. } => params.len(),
        }
    }

    pub fn bind(&self, instance: Object) -> Self {
        match self {
            Function::Native { .. } => unreachable!(),
            Function::User {
                name,
                params,
                body,
                closure,
                is_initializer,
            } => {
                let environment = Rc::new(RefCell::new(Environment::from(closure)));
                environment.borrow_mut().define(
                    &Token {
                        r#type: TokenType::This,
                        lexeme: "this".to_string(),
                        literal: Some(Object::Identifier("this".to_string())),
                        line: 0,
                    },
                    instance,
                );
                Function::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: environment,
                    is_initializer: *is_initializer,
                }
            }
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native function>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
