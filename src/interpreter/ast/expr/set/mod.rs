use crate::interpreter::ast::expr::get::GetType;
use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub enum SetType<T: 'static> {
    Name(Token),
    Index(Token, Box<dyn Expr<T>>),
}

impl<T> From<GetType<T>> for SetType<T> {
    fn from(value: GetType<T>) -> Self {
        match value {
            GetType::Name(name) => SetType::Name(name),
            GetType::Index(token, index) => SetType::Index(token, index),
        }
    }
}

#[derive(Clone)]
pub struct Set<T: 'static> {
    id: u64,
    ty: SetType<T>,
    obj: Box<dyn Expr<T>>,
    value: Box<dyn Expr<T>>,
}

impl<T> Set<T> {
    pub fn new(ty: SetType<T>, obj: Box<dyn Expr<T>>, value: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            ty,
            obj,
            value,
        }
    }

    pub fn extract(&self) -> (&SetType<T>, &dyn Expr<T>, &dyn Expr<T>) {
        (&self.ty, self.obj.deref(), self.value.deref())
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
