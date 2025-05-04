use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;
use std::ops::Deref;

#[derive(Clone)]
pub struct Export<T: 'static> {
    name: Token,
    stmt: Box<dyn Stmt<T>>,
}

impl<T> Export<T> {
    pub fn new(name: Token, stmt: Box<dyn Stmt<T>>) -> Self {
        Self { name, stmt }
    }

    pub fn extract(&self) -> (&Token, &dyn Stmt<T>) {
        (&self.name, self.stmt.deref())
    }
}

impl<T: 'static + Clone> Stmt<T> for Export<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_export(self)
    }
}
