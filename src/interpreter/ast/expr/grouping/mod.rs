use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::utils::next_id;
use std::ops::Deref;

#[derive(Clone)]
pub struct Grouping<T: 'static> {
    id: u64,
    expression: Box<dyn Expr<T>>,
}

impl<T> Grouping<T> {
    #[inline]
    pub fn new(expression: Box<dyn Expr<T>>) -> Self {
        Self {
            id: next_id(),
            expression,
        }
    }

    #[inline]
    pub fn get_expr(&self) -> &dyn Expr<T> {
        self.expression.deref()
    }
}

impl<T: 'static + Clone> Expr<T> for Grouping<T> {
    #[inline]
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_grouping(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
