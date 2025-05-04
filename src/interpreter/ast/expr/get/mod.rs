use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub struct Get<T: 'static> {
    id: u64,
    name: Token,
    object: Box<dyn Expr<T>>,
}

impl<T> Get<T> {
    pub fn new(name: Token, object: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            name,
            object,
        }
    }

    pub fn extract(&self) -> (&Token, &dyn Expr<T>) {
        (&self.name, self.object.deref())
    }
}

impl<T: 'static + Clone> Expr<T> for Get<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_get(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
