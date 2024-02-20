use std::collections::HashMap;

use crate::{
    expression::{Expr, ExprVisitor},
    interpreter::Interpreter,
    returns::Return,
    statement::{Stmt, StmtVisitor},
    Token,
};

struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    fn resolve_stmts(&self, statements: Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, stmt: Stmt) {
        stmt.accept(self);
    }

    fn resolve_expr(&mut self, expr: Expr) {
        expr.accept(self);
    }

    fn resolve_local(&mut self, name: Token) {
        for i in (0..=self.scopes.len()).rev() {
            if self.scopes.get(i).unwrap().contains_key(&name.lexeme) {
                self.interpreter.resolve(name, self.scopes.len() - 1 - i);
            }
        }
    }

    fn resolve_function(&self, name: Token, params: Vec<Token>, body: Vec<Stmt>) {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if !self.scopes.is_empty() {
            if let Some(mut scope) = self.scopes.last_mut() {
                scope.insert(name.lexeme, false);
            }
        }
    }

    fn define(&mut self, name: Token) {
        if !self.scopes.is_empty() {
            if let Some(mut scope) = self.scopes.last_mut() {
                scope.insert(name.lexeme, true);
            }
        }
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_block_stmt(&mut self, statements: Vec<Stmt>) -> Result<(), Return> {
        self.begin_scope();
        self.resolve_stmts(statements);
        self.end_scope();
        Ok(())
    }

    fn visit_var_stmt(
        &mut self,
        name: crate::Token,
        initializer: Option<crate::expression::Expr>,
    ) -> Result<(), Return> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expr(init);
        }
        self.define(name);
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Result<(), Return> {
        self.declare(name);
        self.define(name);

        self.resolve_function(name, params, body);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: Expr) -> Result<(), Return> {
        self.resolve_expr(stmt);
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Box<Option<Stmt>>,
    ) -> Result<(), Return> {
        self.resolve_expr(condition);
        self.resolve_stmt(*then_stmt);
        if let Some(else_branch) = *else_stmt {
            self.resolve_stmt(else_branch);
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: Expr) -> Result<(), Return> {
        self.resolve_expr(stmt);
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: Token, value: Option<Expr>) -> Result<(), Return> {
        if let Some(val) = value {
            self.resolve_expr(val);
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Result<(), Return> {
        self.resolve_expr(condition);
        self.resolve_stmt(*body);
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_var_expr(&self, t: Token) -> () {
        if let Some(scope) = self.scopes.last() {}
        if !self.scopes.is_empty() {
            if let Some(val) = self.scopes.last().unwrap().get(&t.lexeme) {
                if val == &false {
                    Error("");
                }
            }
        }

        self.resolve_local(t);
        ()
    }

    fn visit_assign_expr(&mut self, name: Token, value: Box<Expr>) -> () {
        self.resolve_expr(*value);
        self.resolve_local(name);
        ()
    }

    fn visit_binary_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> () {
        self.resolve_expr(*l);
        self.resolve_expr(*r);
        ()
    }

    fn visit_call_expr(&mut self, c: Box<Expr>, paren: Token, args: Vec<Expr>) -> () {
        self.resolve_expr(*c);

        for arg in args {
            self.resolve_expr(arg);
        }

        ()
    }

    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> () {
        self.resolve_expr(*expr);
        ()
    }

    fn visit_literal_expr(&self, literal: crate::Literal) -> () {
        ()
    }

    fn visit_logical_expr(&mut self, l: Box<Expr>, op: Token, r: Box<Expr>) -> () {
        self.resolve_expr(*l);
        self.resolve_expr(*r);
        ()
    }

    fn visit_unary_expr(&mut self, op: Token, r: Box<Expr>) -> () {
        self.resolve_expr(*r);
        ()
    }
}
