use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;

#[derive(Clone)]
pub struct Buffer<T: 'static> {
    name: Token,
    obj: Box<dyn Expr<T>>
}

impl<T> Buffer<T> {
    pub fn new(name: Token, obj: Box<dyn Expr<T>>) -> Self {
        Self { name, obj }
    }

    pub fn extract(&self) -> (&Token, &dyn Expr<T>) {
        (&self.name, self.obj.as_ref())
    }
}

impl<T: 'static + Clone> Stmt<T> for Buffer<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_buffer(self)
    }
}