use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;

#[derive(Clone)]
pub struct SelfExpr {
    id: u64,
    name: Token
}

impl SelfExpr {
    pub fn new(name: Token) -> Self {
        Self { id: next_id(), name }
    }
    
    pub fn get_name(&self) -> Token {
        self.name.clone()
    }
}

impl<T: 'static + Clone> Expr<T> for SelfExpr {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_self(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}