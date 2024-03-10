use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Object;
use crate::token::Token;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
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
                ..
            } => {
                let environment = Rc::new(RefCell::new(Environment::from(closure)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment.borrow_mut().define(param, argument.clone());
                }
                match interpreter.execute_block(body, environment) {
                    Err(Error::Return { value }) => Ok(value),
                    Err(other) => Err(other),
                    Ok(..) => Ok(Object::Nil),
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
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native function>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
