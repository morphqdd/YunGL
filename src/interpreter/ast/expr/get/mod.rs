use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub enum GetType<T: 'static> {
    Name(Token),
    Index(Token, Box<dyn Expr<T>>),
}

#[derive(Clone)]
pub struct Get<T: 'static> {
    id: u64,
    ty: GetType<T>,
    object: Box<dyn Expr<T>>,
}

impl<T> Get<T> {
    pub fn new(ty: GetType<T>, object: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            ty,
            object,
        }
    }

    pub fn extract(&self) -> (&GetType<T>, &dyn Expr<T>) {
        (&self.ty, self.object.deref())
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
