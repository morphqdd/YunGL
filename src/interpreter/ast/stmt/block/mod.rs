use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
#[derive(Clone)]
pub struct Block<T> {
    stmts: Vec<Box<dyn Stmt<T>>>,
}

impl<T: Clone> Block<T> {
    pub fn new(stmts: Vec<Box<dyn Stmt<T>>>) -> Self {
        Self { stmts }
    }

    pub fn get_stmts(&self) -> Vec<&dyn Stmt<T>> {
        self.stmts.iter().map(AsRef::as_ref).collect::<Vec<_>>()
    }
}

impl<T: 'static + Clone> Stmt<T> for Block<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_block(self)
    }
}
