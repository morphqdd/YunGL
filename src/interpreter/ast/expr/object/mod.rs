use std::collections::HashMap;
use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;

#[derive(Clone)]
pub struct Obj<T: 'static> {
    id: u64,
    values: HashMap<Token, Box<dyn Expr<T>>>,
}

impl<T> Obj<T> {
    pub fn new(values: HashMap<Token, Box<dyn Expr<T>>>) -> Self {
        Self { id: next_id(), values }
    }

    pub fn extract(&self) -> &HashMap<Token, Box<dyn Expr<T>>> {
        &self.values
    }
}

impl<T: 'static + Clone> Expr<T> for Obj<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_object(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}