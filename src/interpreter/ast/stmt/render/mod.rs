use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;

#[derive(Clone)]
pub struct Render<T: 'static> {
    name: Token,
    elements: Vec<Box<dyn Expr<T>>>
}

impl<T> Render<T> {
    pub fn new(name: Token, elements: Vec<Box<dyn Expr<T>>>) -> Self {
        Self { name, elements }
    }

    pub fn extract(&self) -> (&Token, Vec<&dyn Expr<T>>) {
        (&self.name, self.elements.iter().map(AsRef::as_ref).collect())
    }
}

impl<T: 'static + Clone> Stmt<T> for Render<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_render(self)
    }
}