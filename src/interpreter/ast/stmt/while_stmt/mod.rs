use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use std::ops::Deref;

#[derive(Clone)]
pub struct While<T: 'static> {
    cond: Box<dyn Expr<T>>,
    stmt: Box<dyn Stmt<T>>,
}

impl<T> While<T> {
    pub fn new(cond: Box<dyn Expr<T>>, stmt: Box<dyn Stmt<T>>) -> Self {
        Self { cond, stmt }
    }

    pub fn extract(&self) -> (&dyn Expr<T>, &dyn Stmt<T>) {
        (self.cond.deref(), self.stmt.deref())
    }
}

impl<T: 'static + Clone> Stmt<T> for While<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_while(self)
    }
}
