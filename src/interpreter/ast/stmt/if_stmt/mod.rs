use std::ops::Deref;
use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};

type ExtractedIf<'a ,T> = (&'a dyn Expr<T>, &'a dyn Stmt<T>, Option<&'a dyn Stmt<T>>);

#[derive(Clone)]
pub struct If<T: 'static> {
    cond: Box<dyn Expr<T>>,
    then_stmt: Box<dyn Stmt<T>>,
    else_stmt: Option<Box<dyn Stmt<T>>>,
}

impl<T> If<T> {
    pub fn new(
        cond: Box<dyn Expr<T>>,
        then_stmt: Box<dyn Stmt<T>>,
        else_stmt: Option<Box<dyn Stmt<T>>>,
    ) -> Self {
        Self {
            cond,
            then_stmt,
            else_stmt,
        }
    }

    pub fn extract(&self) -> ExtractedIf<T> {
        (self.cond.deref(), self.then_stmt.deref(), self.else_stmt.as_deref())
    }
}

impl<T: 'static + Clone> Stmt<T> for If<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_if(self)
    }
}
