use crate::token::*;
use std::collections::HashMap;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: u32,
    current: u32,
    line: u32,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        // TODO: Think about converting keywords to enum
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::And);
        keywords.insert(String::from("class"), TokenType::Class);
        keywords.insert(String::from("else"), TokenType::Else);
        keywords.insert(String::from("false"), TokenType::False);
        keywords.insert(String::from("for"), TokenType::For);
        keywords.insert(String::from("fun"), TokenType::Fun);
        keywords.insert(String::from("if"), TokenType::If);
        keywords.insert(String::from("nil"), TokenType::Nil);
        keywords.insert(String::from("or"), TokenType::Or);
        keywords.insert(String::from("print"), TokenType::Print);
        keywords.insert(String::from("return"), TokenType::Return);
        keywords.insert(String::from("super"), TokenType::Super);
        keywords.insert(String::from("this"), TokenType::This);
        keywords.insert(String::from("true"), TokenType::True);
        keywords.insert(String::from("var"), TokenType::Var);
        keywords.insert(String::from("while"), TokenType::While);
        Scanner {
            source: String::from(source),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::from(""),
            Literal::None,
            self.line,
        ));
        // for token in self.tokens.iter() {
        //     println!("{:?}", token);
        // }
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen);
            }
            ')' => {
                self.add_token(TokenType::RightParen);
            }
            '{' => {
                self.add_token(TokenType::LeftBrace);
            }
            '}' => {
                self.add_token(TokenType::RightBrace);
            }
            ',' => {
                self.add_token(TokenType::Comma);
            }
            '.' => {
                self.add_token(TokenType::Dot);
            }
            '-' => {
                self.add_token(TokenType::Minus);
            }
            '+' => {
                self.add_token(TokenType::Plus);
            }
            ';' => {
                self.add_token(TokenType::Semicolon);
            }
            '*' => {
                self.add_token(TokenType::Star);
            }
            '!' => {
                if self.match_token('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '=' => {
                if self.match_token('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '<' => {
                if self.match_token('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.match_token('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            '/' => {
                if self.match_token('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\t' | '\r' => {
                // Ignore
            }
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string();
            }
            'o' => {
                if self.match_token('r') {
                    self.add_token(TokenType::Or);
                }
            }
            _ => {
                if is_digit(c) {
                    self.number();
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    panic!("Lexical error on {}", self.line)
                }
            }
        };
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source_substring(self.start, self.current);
        let token_type: TokenType = match self.keywords.get(text) {
            Some(x) => x.to_owned(),
            None => TokenType::Identifier,
        };

        self.add_token(token_type);
    }

    fn source_substring(&self, start: u32, current: u32) -> &str {
        let s = self.source.as_str();
        let start_us = usize::try_from(start).unwrap();
        let current_us = usize::try_from(current).unwrap();
        let value = &s[start_us..current_us];
        value
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source_substring(self.start, self.current);
        let number: f64 = value.parse().unwrap();

        self.add_token_full(TokenType::Number, Literal::Number(number));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated string. {}", self.line);
        } else {
            self.advance();

            let value = self.source_substring(self.start + 1, self.current - 1);

            self.add_token_full(TokenType::String, Literal::String(value.to_owned()));
        }
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        let current_us = usize::try_from(self.current).unwrap();
        if self.source.chars().nth(current_us).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        let current_us = usize::try_from(self.current).unwrap();
        return self.source.chars().nth(current_us).unwrap();
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len().try_into().unwrap() {
            return '\0';
        }
        let current_us = usize::try_from(self.current).unwrap();
        return self.source.chars().nth(current_us).unwrap();
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len().try_into().unwrap()
    }

    fn advance(&mut self) -> char {
        let n = self.current;
        self.current += 1;
        let n_us = usize::try_from(n).unwrap();
        self.source.chars().nth(n_us).unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_full(token_type, Literal::None);
    }

    fn add_token_full(&mut self, token_type: TokenType, literal: Literal) {
        let text = self.source_substring(self.start, self.current);
        self.tokens.push(Token::new(
            token_type,
            String::from(text),
            literal,
            self.line,
        ));
    }
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}
