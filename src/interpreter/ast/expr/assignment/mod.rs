use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub struct Assign<T: 'static> {
    id: u64,
    token: Token,
    value: Box<dyn Expr<T>>,
}

impl<T> Assign<T> {
    pub fn new(token: Token, value: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            token,
            value,
        }
    }

    pub fn get_token(&self) -> Token {
        self.token.clone()
    }

    pub fn get_value(&self) -> &dyn Expr<T> {
        self.value.deref()
    }
}

impl<T: 'static + Clone> Expr<T> for Assign<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_assign(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
