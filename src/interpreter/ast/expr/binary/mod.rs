use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::token_type::TokenType;
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub struct Binary<T: 'static> {
    id: u64,
    left: Box<dyn Expr<T>>,
    operation: Token,
    right: Box<dyn Expr<T>>,
}

impl<T> Binary<T> {
    pub fn new(left: Box<dyn Expr<T>>, operation: Token, right: Box<dyn Expr<T>>) -> Binary<T> {
        Binary {
            id: next_id(),
            left,
            operation,
            right,
        }
    }

    #[inline]
    pub fn get_left(&self) -> &dyn Expr<T> {
        self.left.deref()
    }

    #[inline]
    pub fn get_right(&self) -> &dyn Expr<T> {
        self.right.deref()
    }

    #[inline]
    pub fn get_op_lexeme(&self) -> &str {
        self.operation.get_lexeme()
    }

    #[inline]
    pub fn get_op_type(&self) -> TokenType {
        self.operation.get_type()
    }

    #[inline]
    pub fn get_token(&self) -> Token {
        self.operation.clone()
    }
}

impl<T: 'static + Clone> Expr<T> for Binary<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_binary(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
