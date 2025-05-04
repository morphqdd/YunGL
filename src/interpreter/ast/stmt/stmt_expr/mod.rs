use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use std::ops::Deref;

#[derive(Clone)]
pub struct StmtExpr<T: 'static> {
    expr: Box<dyn Expr<T>>,
}

impl<T> StmtExpr<T> {
    pub fn new(expr: Box<dyn Expr<T>>) -> Self {
        Self { expr }
    }

    pub fn expr(&self) -> &dyn Expr<T> {
        self.expr.deref()
    }
}

impl<T: 'static + Clone> Stmt<T> for StmtExpr<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_expr(self)
    }
}
