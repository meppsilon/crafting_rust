use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Interpreter, Value},
    returns::Return,
    statement::{Stmt, StmtVisitor},
    Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Value>) -> Value>,
    },
    User {
        // arity: usize,
        body: Vec<Stmt>,
        params: Vec<Token>,
        name: Token,
        enclosing: Rc<RefCell<Environment>>,
    },
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native fn>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}

impl Function {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        match self {
            Function::Native { body, .. } => Ok(body(&arguments)),
            Function::User { params, body, enclosing, .. } => {
                let mut env = Environment::new_from(enclosing);
                for i in 0..params.len() {
                    env.define(params[i].lexeme.clone(), arguments[i].clone());
                }

                match interpreter.execute_block(body.clone(), Rc::new(RefCell::new(env))) {
                    Err(Return { value }) => {
                        Ok(value)
                    }
                    Ok(..) => Ok(Value::None),
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
