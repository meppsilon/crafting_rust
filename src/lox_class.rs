use std::collections::HashMap;

use crate::{interpreter::Value, Token};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
}

impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub klass: LoxClass,
    pub fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn get(&self, name: Token) -> Value {
        if let Some(value) = self.fields.get(&name.lexeme) {
            *value
        } else {
            Value::None
        }
    }

    pub fn set(&mut self, name: Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}
