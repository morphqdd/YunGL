use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::utils::next_id;

#[derive(Clone)]
pub struct List<T: 'static> {
    id: u64,
    values: Vec<Box<dyn Expr<T>>>,
}

impl<T> List<T> {
    pub fn new(values: Vec<Box<dyn Expr<T>>>) -> Self {
        Self { id: next_id(), values }
    }
    
    pub fn extract_values(&self) -> Vec<&dyn Expr<T>> {
        self.values.iter().map(AsRef::as_ref).collect()
    }
}

impl<T: 'static + Clone> Expr<T> for List<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_list(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}