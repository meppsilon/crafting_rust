use crate::{
    interpreter::{Interpreter, Value},
    token::*,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Grouping(Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    // Get(Box<Expr>, String),
    Variable(Token),
    Literal(Literal),
    // This,
    // Super,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(t) => write!(f, "{t}"),
            Expr::Grouping(e) => write!(f, "{e}"),
            Expr::Binary(l, op, r) => write!(f, "({l} {op} {r})"),
            Expr::Variable(s) => write!(f, "{s}"),
            Expr::Assign(l, r) => write!(f, "{l} = {r}"),
            Expr::Logical(l, op, r) => write!(f, "({l} {op} {r}"),
            Expr::Call(c, _, args) => write!(f, "{c}({})", itertools::join(args, ", ")),
            // Expr::Get(from, name) => write!(f, "{from}.{name}"),
            Expr::Unary(op, r) => write!(f, "({op}{r})"),
            // Expr::This => write!(f, "this"),
            // Expr::Super => write!(f, "super"),
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut Interpreter) -> Value {
        match self {
            Expr::Grouping(g) => visitor.visit_grouping_expr(g.clone()),
            Expr::Literal(l) => visitor.visit_literal_expr(l.clone()),
            Expr::Unary(op, r) => visitor.visit_unary_expr(op.clone(), r.clone()),
            Expr::Binary(l, op, r) => visitor.visit_binary_expr(l.clone(), op.clone(), r.clone()),
            Expr::Variable(t) => visitor.visit_var_expr(t.clone()),
            Expr::Assign(l, r) => visitor.visit_assign_expr(l.clone(), r.clone()),
            Expr::Logical(l, op, r) => visitor.visit_logical_expr(l.clone(), op.clone(), r.clone()),
            Expr::Call(c, paren, args) => visitor.visit_call_expr(c.clone(), paren.clone(), args.clone()),
        }
    }
    pub fn assign(lvalue: Token, rvalue: Expr) -> Self {
        Self::Assign(lvalue, Box::new(rvalue))
    }

    pub fn binary(left: Expr, operator: Token, right: Expr) -> Self {
        Self::Binary(Box::new(left), operator, Box::new(right))
    }

    pub fn unary(operator: Token, right: Expr) -> Self {
        Self::Unary(operator, Box::new(right))
    }

    pub fn group(self) -> Self {
        Self::Grouping(Box::new(self))
    }

    pub fn literal(value: Literal) -> Self {
        Self::Literal(value)
    }

    pub fn logical(left: Expr, operator: Token, right: Expr) -> Self {
        Self::Logical(Box::new(left), operator, Box::new(right))
    }

    pub fn variable(t: Token) -> Self {
        Self::Variable(t)
    }

    pub fn call(callee: Expr, paren: Token, args: Vec<Expr>) -> Self {
        Self::Call(Box::new(callee), paren, args)
    }
}

pub trait ExprVisitor<T> {
    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> T;
    fn visit_literal_expr(&self, literal: Literal) -> T;
    fn visit_unary_expr(&mut self, op: Token, r: Box<Expr>) -> T;
    fn visit_binary_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> T;
    fn visit_var_expr(&self, t: Token) -> T;
    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> T;
    fn visit_logical_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> T;
    fn visit_call_expr(&mut self, c: Box<Expr>, paren: Token, args: Vec<Expr>) -> T;
}
