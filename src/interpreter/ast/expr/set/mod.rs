use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub struct Set<T: 'static> {
    id: u64,
    name: Token,
    obj: Box<dyn Expr<T>>,
    value: Box<dyn Expr<T>>,
}

impl<T> Set<T> {
    pub fn new(name: Token, obj: Box<dyn Expr<T>>, value: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            name,
            obj,
            value,
        }
    }

    pub fn extract(&self) -> (&Token, &dyn Expr<T>, &dyn Expr<T>) {
        (&self.name, self.obj.deref(), self.value.deref())
    }
}

impl<T: 'static + Clone> Expr<T> for Set<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_set(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
