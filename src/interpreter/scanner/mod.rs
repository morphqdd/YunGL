pub mod token;

use crate::interpreter::error::Result;
use crate::interpreter::object::Object;
use crate::interpreter::scanner::error::{ScannerError, ScannerErrorType};
use crate::interpreter::scanner::token::Token;
use crate::interpreter::scanner::token::token_type::TokenType;
use std::collections::HashMap;

pub mod error;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    pos_in_line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and".into(), TokenType::And);
        keywords.insert("or".into(), TokenType::Or);

        keywords.insert("if".into(), TokenType::If);
        keywords.insert("else".into(), TokenType::Else);

        keywords.insert("true".into(), TokenType::True);
        keywords.insert("false".into(), TokenType::False);

        keywords.insert("for".into(), TokenType::For);
        keywords.insert("while".into(), TokenType::While);

        keywords.insert("fun".into(), TokenType::Fun);
        keywords.insert("class".into(), TokenType::Class);
        keywords.insert("let".into(), TokenType::Let);

        keywords.insert("nil".into(), TokenType::Nil);

        keywords.insert("print".into(), TokenType::Print);
        keywords.insert("return".into(), TokenType::Return);

        keywords.insert("super".into(), TokenType::Super);
        keywords.insert("self".into(), TokenType::Slf);

        keywords.insert("use".into(), TokenType::Use);
        keywords.insert("export".into(), TokenType::Export);

        Self {
            source: source.into(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            pos_in_line: 1,
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(
            TokenType::Eof,
            "",
            None,
            self.line,
            self.pos_in_line + 1,
        ));

        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '[' => self.add_token(TokenType::LeftBracket, None),
            ']' => self.add_token(TokenType::RightBracket, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            ':' => self.add_token(TokenType::Colon, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' if self.find_match('=') => self.add_token(TokenType::BangEqual, None),
            '!' => self.add_token(TokenType::Bang, None),
            '=' if self.find_match('=') => self.add_token(TokenType::EqualEqual, None),
            '=' => self.add_token(TokenType::Equal, None),
            '<' if self.find_match('=') => self.add_token(TokenType::LessEqual, None),
            '<' => self.add_token(TokenType::Less, None),
            '>' if self.find_match('=') => self.add_token(TokenType::GreaterEqual, None),
            '>' => self.add_token(TokenType::Greater, None),
            '/' if self.find_match('/') => {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash, None),
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
                self.pos_in_line = 1;
            }
            '"' => self.string()?,
            _ => {
                if self.is_digit(c) {
                    self.number()?
                } else if self.is_alpha(c) {
                    self.identifier()?
                } else {
                    return Err(ScannerError::new(
                        self.line,
                        self.pos_in_line,
                        ScannerErrorType::UnexpectedCharacter(c),
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    fn advance(&mut self) -> char {
        let ch = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        self.pos_in_line += 1;
        ch
    }

    fn add_token(&mut self, ty: TokenType, lit: Option<Object>) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(
            ty,
            &text,
            lit,
            self.line,
            self.pos_in_line - text.split("\n").nth(0).unwrap().len(),
        ));
    }

    fn find_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn string(&mut self) -> Result<()> {
        let mut ch = self.peek();
        while ch != '"' && !self.is_at_end() {
            if ch == '\n' {
                self.line += 1;
                self.pos_in_line = 1;
            }
            self.advance();
            ch = self.peek();
            //println!("character: {} {}", ch, ch_);
        }

        if self.is_at_end() {
            return Err(ScannerError::new(
                self.line,
                self.pos_in_line,
                ScannerErrorType::UnterminatedString,
            )
            .into());
        }

        self.advance();
        let value = self.source[self.start + 1..self.current - 1].replace("\\n", "\n");
        self.add_token(TokenType::String, Some(Object::String(value)));
        Ok(())
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) -> Result<()> {
        let mut ch = self.peek();
        while self.is_digit(ch) {
            self.advance();
            ch = self.peek();
        }

        let ch_n = self.peek_next();

        if self.peek() == '.' && self.is_digit(ch_n) {
            self.advance();

            let mut ch = self.peek();
            while self.is_digit(ch) {
                self.advance();
                ch = self.peek();
            }
        }
        //print!("[{}:{}]", self.line, self.pos_in_line);
        //println!("number: {}", self.source[self.start..self.current].to_string());
        self.add_token(
            TokenType::Number,
            Some(Object::Number(
                self.source[self.start..self.current]
                    .to_string()
                    .parse()
                    .map_err(|e| {
                        panic!("{e}");
                    })
                    .unwrap(),
            )),
        );
        Ok(())
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn identifier(&mut self) -> Result<()> {
        let mut ch = self.peek();
        while self.is_alphanumeric(ch) {
            self.advance();
            ch = self.peek();
        }

        let text = self.source[self.start..self.current].to_string();
        match self.keywords.get(&text) {
            None => self.add_token(TokenType::Identifier, None),
            Some(ty) => self.add_token(ty.clone(), None),
        }
        Ok(())
    }
}
