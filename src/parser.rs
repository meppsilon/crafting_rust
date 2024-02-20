use crate::expression::*;
use crate::statement::Stmt;
use crate::token::*;
use std::result::Result;

pub struct Parser {
    tokens: Vec<Token>,
    current: u32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            if let Some(dec) = self.declaration() {
                statements.push(dec);
            }
        }

        statements
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::For]) {
            self.for_statement()
        } else if self.match_token(vec![TokenType::If]) {
            self.if_statement()
        } else if self.match_token(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(vec![TokenType::While]) {
            self.while_statement()
        } else if self.match_token(vec![TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer: Option<Stmt> = if self.match_token(vec![TokenType::Semicolon]) {
            None
        } else if self.match_token(vec![TokenType::Var]) {
            self.var_declaration().ok()
        } else {
            self.expression_statement().ok()
        };

        let condition = if self.check(TokenType::Semicolon) {
            Expr::literal(Literal::Boolean(true))
        } else {
            self.expression()?
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment: Option<Expr> = if self.check(TokenType::RightParen) {
            None
        } else {
            self.expression().ok()
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }

        body = Stmt::While(condition, Box::new(body));

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(vec![TokenType::Else]) {
            self.statement().ok()
        } else {
            None
        };

        Ok(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::Semicolon) {
            if let Ok(ex) = self.expression() {
                Some(ex)
            } else {
                None
            }
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.");
        Ok(Stmt::Return(keyword, value))
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = if self.match_token(vec![TokenType::Equal]) {
            self.expression().ok()
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(name, initializer))
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn function(&mut self, kind: String) -> Result<Stmt, String> {
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {} name.", kind))
            .unwrap();
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    crate::error_at_token(self.peek(), "Can't have more than 255 parameters");
                }

                if let Ok(param) = self.consume(TokenType::Identifier, "Expect parameter name.") {
                    parameters.push(param);
                }

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block();
        Ok(Stmt::Function(name, parameters, body))
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(dec) = self.declaration() {
                statements.push(dec);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")
            .unwrap();
        statements
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(vec![TokenType::Equal]) {
            let equals = self.previous().clone();
            let value: Expr = self.assignment()?;
            if let Expr::Variable(token) = expr {
                return Ok(Expr::assign(token, value));
            } else if let Expr::Get(get, name) = expr {
                return Ok(Expr::set(*get, name, value));
            }
            return Err(format!("Invalid assignment target. {}", equals));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(vec![TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(vec![TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(vec![TokenType::Class]) {
            if let Ok(stmt) = self.class_declaration() {
                Some(stmt)
            } else {
                None
            }
        } else if self.match_token(vec![TokenType::Fun]) {
            if let Ok(f) = self.function("function".to_owned()) {
                Some(f)
            } else {
                None
            }
        } else if self.match_token(vec![TokenType::Var]) {
            if let Ok(stmt) = self.var_declaration() {
                Some(stmt)
            } else {
                self.synchronize();
                None
            }
        } else if let Ok(stmt) = self.statement() {
            Some(stmt)
        } else {
            self.synchronize();
            None
        }
    }

    fn class_declaration(&self) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Ok(stmt) = self.function("method".to_owned()) {
                methods.push(stmt);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.");

        Ok(Stmt::Class(name, methods))
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.comparison()?;

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::unary(operator, right))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr);
            } else if self.match_token(vec![TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::get(expr, name);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    crate::error_at_token(self.peek(), "Can't have more than 255 arguments.");
                }
                if let Ok(x) = self.expression() {
                    arguments.push(x);
                }

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self
            .consume(TokenType::RightParen, "Expect ')' after arguments.")
            .unwrap();

        Expr::call(callee, paren, arguments)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(vec![TokenType::False]) {
            return Ok(Expr::literal(Literal::Boolean(false)));
        }
        if self.match_token(vec![TokenType::True]) {
            return Ok(Expr::literal(Literal::Boolean(true)));
        }
        if self.match_token(vec![TokenType::Nil]) {
            return Ok(Expr::literal(Literal::None));
        }

        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::literal(self.previous().clone().literal));
        }

        if self.match_token(vec![TokenType::Identifier]) {
            return Ok(Expr::variable(self.previous().clone()));
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::group(expr));
        }
        crate::error_at_token(self.peek(), "Expect expression");
        Err("Parser error".to_string())
    }

    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == token_type;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        let current_us = usize::try_from(self.current).unwrap();
        &self.tokens[current_us]
    }

    fn previous(&self) -> &Token {
        let current_us = usize::try_from(self.current).unwrap();
        &self.tokens[current_us - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            crate::error_at_token(self.peek(), message);
            Err("Parse error".to_string())
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }
}
