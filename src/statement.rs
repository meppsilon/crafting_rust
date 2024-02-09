use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment, expression::Expr, interpreter::Interpreter, returns::Return,
    token::Token,
};

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Option<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(Token, Vec<Token>, Vec<Stmt>),
    Return(Token, Option<Expr>),
}

impl Stmt {
    pub fn accept(&self, visitor: &mut Interpreter) -> Result<(), Return> {
        match self {
            Self::Expression(e) => visitor.visit_expression_stmt(e.clone()),
            Self::Print(e) => visitor.visit_print_stmt(e.clone()),
            Self::Var(t, e) => visitor.visit_var_stmt(t.clone(), e.clone()),
            Self::Block(stmts) => visitor.visit_block_stmt(stmts.clone()),
            Self::If(condition, then_stmt, else_stmt) => {
                visitor.visit_if_stmt(condition.clone(), then_stmt.clone(), else_stmt.clone())
            }
            Self::While(condition, body) => {
                visitor.visit_while_stmt(condition.clone(), body.clone())
            }
            Self::Function(name, params, body) => {
                visitor.visit_function_stmt(name.clone(), params.clone(), body.clone())
            }
            Self::Return(keyword, value) => {
                visitor.visit_return_stmt(keyword.clone(), value.clone())
            }
        }
    }
}

pub trait StmtVisitor<T> {
    fn execute(&mut self, stmt: Stmt) -> Result<(), Return>;
    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Return>;
    fn visit_expression_stmt(&mut self, stmt: Expr) -> Result<(), Return>;
    fn visit_print_stmt(&mut self, stmt: Expr) -> Result<(), Return>;
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Result<(), Return>;
    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<(), Return>;
    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Box<Option<Stmt>>,
    ) -> Result<(), Return>;
    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<(), Return>;
    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<(), Return>;
    fn visit_return_stmt(&mut self, keyword: Token, value: Option<Expr>) -> Result<(), Return>;
}
