use crate::environment::Environment;
use crate::expression::{Expr, ExprVisitor};
use crate::function::Function;
use crate::lox_class::{LoxClass, LoxInstance};
use crate::returns::Return;
use crate::statement::{Stmt, StmtVisitor};
use crate::token::{Literal, Token, TokenType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(Function),
    Class(LoxClass),
    Instance(LoxInstance),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::None => write!(f, "none"),
            Self::Callable(_) => write!(f, "function"),
            Self::Class(_) => write!(f, "class"),
            Self::Instance(_) => write!(f, "instance"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (_, Value::None) => false,
            (Value::None, _) => false,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left.eq(right),
            _ => false, // TODO: this should be defined or all.
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    // locals: HashMap<Expr, usize>,
    pub globals: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        let clock = Value::Callable(Function::Native {
            arity: 0,
            body: Box::new(|_args: &Vec<Value>| {
                Value::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        });

        globals.define("clock".to_string(), clock);
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
            globals,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.execute(statement);
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Value {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), Return> {
        stmt.accept(self)?;
        Ok(())
    }

    // fn resolve(&mut self, expr: Expr, depth: int) {
    //     self.locals.put
    // }

    pub fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Return> {
        let previous = self.environment.clone();
        let steps = || -> Result<(), Return> {
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
}

impl ExprVisitor<Value> for Interpreter {
    fn visit_literal_expr(&self, literal: Literal) -> Value {
        match literal {
            Literal::None => Value::None,
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s.clone()),
        }
    }

    fn visit_logical_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> Value {
        let left = self.evaluate(*l);

        if op.token_type == TokenType::Or {
            if is_truthy(&left) {
                return left;
            }
        } else if !is_truthy(&left) {
            return left;
        }

        self.evaluate(*r)
    }

    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> Value {
        self.evaluate(*expr)
    }

    fn visit_unary_expr(&mut self, op: Token, r: Box<Expr>) -> Value {
        let right = self.evaluate(*r);

        match op.token_type {
            TokenType::Bang => Value::Boolean(!is_truthy(&right)),
            TokenType::Minus => {
                if let Value::Number(n) = right {
                    Value::Number(-n)
                } else {
                    panic!("{:?} must be a number", right);
                }
            }
            _ => Value::None,
        }
    }

    fn visit_binary_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> Value {
        let left = self.evaluate(*l);
        let right = self.evaluate(*r);

        match op.token_type {
            TokenType::Minus => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Number(ln - rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::Slash => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Number(ln / rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::Star => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Number(ln * rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::Plus => match (left.clone(), right.clone()) {
                (Value::String(ls), Value::String(rs)) => Value::String(ls + &rs),
                (Value::Number(ln), Value::Number(rn)) => Value::Number(ln + rn),
                _ => panic!(
                    "{:?} and {:?} must both be strings or both be numbers",
                    left, right
                ),
            },
            TokenType::Greater => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Boolean(ln > rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::GreaterEqual => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Boolean(ln >= rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::Less => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Boolean(ln < rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::LessEqual => {
                if let (Value::Number(ln), Value::Number(rn)) = (left.clone(), right.clone()) {
                    Value::Boolean(ln <= rn)
                } else {
                    panic!("{:?} and {:?} must be numbers", left, right);
                }
            }
            TokenType::BangEqual => Value::Boolean(!is_equal(left, right)),
            TokenType::EqualEqual => Value::Boolean(is_equal(left, right)),
            _ => Value::None,
        }
    }

    fn visit_var_expr(&self, name: Token) -> Value {
        let value = self.environment.borrow().get(&name).unwrap();
        value
    }

    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> Value {
        let value = self.evaluate(*value);
        self.environment
            .borrow_mut()
            .assign(name, value.clone())
            .unwrap();
        value
    }

    fn visit_call_expr(&mut self, c: Box<Expr>, paren: Token, args: Vec<Expr>) -> Value {
        let callee = self.evaluate(*c);

        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(self.evaluate(arg));
        }

        match callee {
            Value::Callable(function) => {
                if arguments.len() != function.arity() {
                    crate::error_at_token(
                        &paren,
                        &format!(
                            "Expected {} argument but got {}.",
                            function.arity(),
                            arguments.len()
                        ),
                    );
                }
                if let Ok(res) = function.call(self, arguments) {
                    res
                } else {
                    Value::None
                }
            }
            Value::Class(class) => {
                let args_size = args.len();
                let instance = LoxInstance { klass: class, fields: HashMap::new() };
                Value::None
            }
            _ => {
                crate::error_at_token(&paren, "Can only call functions and classes.");
                Value::None
            }
        }
    }

    fn visit_get_expr(&mut self, expr: Box<Expr>, name: Token) -> Result<Value, String> {
        let value = self.evaluate(*expr);
        if let Value::Instance(instance) = value {
            Ok(instance.get(name))
        } else {
            Err("Only instances have properties".to_string())
        }
    }

    fn visit_set_expr(&mut self, object: Box<Expr>, name: Token, value: Box<Expr>) -> Result<Value, String> {
        let object_value = self.evaluate(*object);

        if let Value::Instance(mut instance) = object_value {
            let value_value = self.evaluate(*value);
            instance.set(name, value_value);
            Ok(value_value)
        } else {
            Err("Only instances have fields.".to_string())
        }
    }
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: Expr) -> Result<(), Return> {
        self.evaluate(stmt);
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: Expr) -> Result<(), Return> {
        let value = self.evaluate(stmt);
        println!("{}", value);
        Ok(())
    }

    fn visit_return_stmt(&mut self, _: Token, value: Option<Expr>) -> Result<(), Return> {
        let value_value = if let Some(v) = value {
            self.evaluate(v)
        } else {
            Value::None
        };

        Err(Return { value: value_value })
    }

    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Result<(), Return> {
        let value = initializer.map_or_else(|| Value::None, |expr| self.evaluate(expr));
        self.environment.borrow_mut().define(name.lexeme, value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<(), Return> {
        while is_truthy(&self.evaluate(condition.clone())) {
            self.execute(*body.clone())?;
        }
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<(), Return> {
        let environment = Environment::new_from(&self.environment);
        self.execute_block(statements, Rc::new(RefCell::new(environment)))?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Result<(), Return> {
        if is_truthy(&self.evaluate(condition)) {
            self.execute(*then_branch)?;
        } else if let Some(e) = *else_branch {
            self.execute(e)?;
        }
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<(), Return> {
        let key = name.lexeme.clone();
        let function = Value::Callable(Function::User {
            body,
            params,
            name,
            enclosing: Rc::clone(&self.environment),
        });
        self.environment.borrow_mut().define(key, function);
        Ok(())
    }

    fn visit_class_stmt(&self, name: Token, methods: Vec<Stmt>) -> Result<(), Return> {
        self.environment
            .borrow_mut()
            .define(name.lexeme, Value::None);
        let klass = LoxClass { name: name.lexeme };
        self.environment
            .borrow_mut()
            .assign(name, Value::Class(klass));
        Ok(())
    }
}

fn is_truthy(object: &Value) -> bool {
    match object {
        Value::None => false,
        Value::Boolean(b) => *b,
        _ => true,
    }
}

fn is_equal(left: Value, right: Value) -> bool {
    match (left, right) {
        (Value::None, Value::None) => true,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Number(l), Value::Number(r)) => l == r,
        (Value::String(l), Value::String(r)) => l == r,
        _ => false,
    }
}

// fn stringify(object: Value) -> String {
//     match object {
//         Value::None => "none".to_string(),
//         Value::Boolean(b) => b.to_string(),
//         Value::Number(n) => n.to_string(),
//         Value::String(s) => s,
//     }
// }
