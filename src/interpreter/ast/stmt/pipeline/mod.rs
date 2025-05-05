use crate::interpreter::ast::expr::{Expr};
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;

#[derive(Clone)]
pub struct Pipeline<T: 'static> {
    name: Token,
    obj: Box<dyn Expr<T>>,
}

impl<T> Pipeline<T> {
    pub fn new(name: Token, obj: Box<dyn Expr<T>>) -> Self {
        Self { name, obj }
    }

    pub fn extract(&self) -> (&Token, &dyn Expr<T>) {
        (&self.name, self.obj.as_ref())
    }
}

impl<T: 'static + Clone> Stmt<T> for Pipeline<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_pipeline(self)
    }
}