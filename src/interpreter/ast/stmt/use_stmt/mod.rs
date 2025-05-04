use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;
use std::ops::Deref;

#[derive(Clone)]
pub struct Use<T: 'static> {
    name: Token,
    expr: Box<dyn Expr<T>>,
}

impl<T> Use<T> {
    pub fn new(name: Token, expr: Box<dyn Expr<T>>) -> Self {
        Self { name, expr }
    }

    pub fn extract(&self) -> (&Token, &dyn Expr<T>) {
        (&self.name, self.expr.deref())
    }
}

impl<T: 'static + Clone> Stmt<T> for Use<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_use(self)
    }
}
