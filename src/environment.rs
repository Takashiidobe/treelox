use crate::error::Error;
use crate::token::{Object, Token};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>, // Parent
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &Token, value: Object) {
        let key = &*name.lexeme;
        self.values.insert(key.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, Error> {
        let key = &*name.lexeme;
        if let Some(value) = self.values.get(key) {
            Ok((*value).clone())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(Error::Runtime {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", key),
            })
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), Error> {
        let key = &*name.lexeme;
        if self.values.contains_key(key) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(ref enclosing) = self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(Error::Runtime {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", key),
            })
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let rc = self
            .enclosing
            .clone()
            .unwrap_or_else(|| panic!("No enclosing environment at {}", 1));
        let parent = rc;
        let mut environment = Rc::clone(&parent);

        for i in 1..distance {
            let parent = environment
                .borrow()
                .enclosing
                .clone()
                .unwrap_or_else(|| panic!("No enclosing environment at {}", i));
            environment = Rc::clone(&parent);
        }
        environment
    }

    pub(crate) fn get_at(&self, distance: usize, name: &Token) -> Result<Object, Error> {
        let key = &*name.lexeme;
        if distance > 0 {
            Ok(self
                .ancestor(distance)
                .borrow()
                .values
                .get(key)
                .unwrap_or_else(|| panic!("Undefined variable '{}'", key))
                .clone())
        } else {
            Ok(self
                .values
                .get(key)
                .unwrap_or_else(|| panic!("Undefined variable '{}'", key))
                .clone())
        }
    }

    pub(crate) fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), Error> {
        if distance > 0 {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.lexeme.clone(), value);
        } else {
            self.values.insert(name.lexeme.clone(), value);
        }
        Ok(())
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "values: {:?}", self.values)
    }
}
