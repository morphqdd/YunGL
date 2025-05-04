use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;

#[derive(Clone)]
pub struct Super {
    id: u64,
    keyword: Token,
    method: Token,
}

impl Super {
    pub fn new(keyword: Token, method: Token) -> Self {
        Self {
            id: next_id(),
            keyword,
            method,
        }
    }

    pub fn extract(&self) -> (Token, Token) {
        (self.keyword.clone(), self.method.clone())
    }
}

impl<T> Expr<T> for Super {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_super(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
