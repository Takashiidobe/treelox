use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    error::Error,
    function::Function,
    token::{Object, Token},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Function>,
}

impl Class {
    pub fn find_method(&self, name: &str) -> Option<&Function> {
        self.methods.get(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: Rc<RefCell<Class>>,
    fields: HashMap<Token, Object>,
}

impl Instance {
    pub fn new(class: &Rc<RefCell<Class>>) -> Object {
        let instance = Instance {
            class: Rc::clone(class),
            fields: HashMap::new(),
        };
        Object::Instance(Rc::new(RefCell::new(instance)))
    }

    pub fn get(&self, name: &Token, instance: &Object) -> Result<Object, Error> {
        if let Some(field) = self.fields.get(&name) {
            Ok(field.clone())
        } else if let Some(method) = self.class.borrow().find_method(&name.lexeme) {
            Ok(Object::Callable(method.bind(instance.clone())))
        } else {
            Err(Error::Runtime {
                token: name.clone(),
                message: format!("Undefined property '{}'.", name.lexeme),
            })
        }
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.clone(), value);
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
