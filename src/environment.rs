use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::Value, token::Token};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(env) = &self.enclosing {
            write!(
                f,
                "values: {}, enclosing: {}",
                self.values
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .join(", "),
                env.borrow().to_string(),
            )
        } else {
            write!(
                f,
                "values: {}",
                self.values
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .join(", "),
            )
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, key: String, value: Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Token) -> Result<Value, String> {
        if let Some(value) = self.values.get(&key.lexeme) {
            Ok(value.clone())
        } else if let Some(env) = &self.enclosing {
            env.borrow().get(key)
        } else {
            Err(format!("Undefined variable '{}'.", key.lexeme))
        }
    }

    pub fn assign(&mut self, name: Token, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(env) = &self.enclosing {
            env.borrow_mut().assign(name, value)?;
            Ok(())
        } else {
            Err(format!("Undefined variable {}.", name.lexeme))
        }
    }
}
