use crate::interpreter::object::Object;
use crate::interpreter::scanner::token::token_type::TokenType;
use std::fmt::{Display, Formatter};

pub mod token_type;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Object>,
    line: usize,
    pos_in_line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: &str,
        literal: Option<Object>,
        line: usize,
        pos_in_line: usize,
    ) -> Token {
        Self {
            token_type,
            lexeme: lexeme.into(),
            literal,
            line,
            pos_in_line,
        }
    }

    pub fn builtin_void(token_type: TokenType, lexeme: &str, literal: Option<Object>) -> Self {
        Self {
            token_type,
            lexeme: lexeme.into(),
            literal,
            line: 0,
            pos_in_line: 0,
        }
    }

    pub fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type.clone()
    }

    pub fn get_lit(&self) -> Option<Object> {
        self.literal.clone()
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_pos_in_line(&self) -> usize {
        self.pos_in_line
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}
