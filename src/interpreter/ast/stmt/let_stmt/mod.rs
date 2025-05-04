use crate::interpreter::ast::expr::Expr;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;
use std::ops::Deref;

#[derive(Clone)]
pub struct Let<T: 'static> {
    ident: Token,
    initializer: Option<Box<dyn Expr<T>>>,
}

impl<T> Let<T> {
    pub fn new(ident: Token, initializer: Option<Box<dyn Expr<T>>>) -> Self {
        Self { ident, initializer }
    }

    pub fn get_ident(&self) -> Token {
        self.ident.clone()
    }

    pub fn get_initializer(&self) -> Option<&dyn Expr<T>> {
        if let Some(initializer) = &self.initializer {
            return Some(initializer.deref());
        }
        None
    }
}

impl<T: 'static + Clone> Stmt<T> for Let<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_let(self)
    }
}
