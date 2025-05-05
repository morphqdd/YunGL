use crate::interpreter::ast::stmt::block::Block;
use crate::interpreter::ast::stmt::class::Class;
use crate::interpreter::ast::stmt::export_stmt::Export;
use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::ast::stmt::if_stmt::If;
use crate::interpreter::ast::stmt::let_stmt::Let;
use crate::interpreter::ast::stmt::print::Print;
use crate::interpreter::ast::stmt::return_stmt::Return;
use crate::interpreter::ast::stmt::stmt_expr::StmtExpr;
use crate::interpreter::ast::stmt::use_stmt::Use;
use crate::interpreter::ast::stmt::while_stmt::While;
use downcast_rs::{impl_downcast, Downcast};
use crate::interpreter::ast::stmt::buffer::Buffer;
use crate::interpreter::ast::stmt::pipeline::Pipeline;

pub mod block;
pub mod class;
pub mod export_stmt;
pub mod fun_stmt;
pub mod if_stmt;
pub mod let_stmt;
pub mod print;
pub mod return_stmt;
pub mod stmt_expr;
pub mod use_stmt;
pub mod while_stmt;
pub mod pipeline;
pub mod buffer;
pub mod render;

pub trait StmtVisitor<T> {
    fn visit_expr(&mut self, stmt: &StmtExpr<T>) -> T;
    fn visit_print(&mut self, stmt: &Print<T>) -> T;
    fn visit_let(&mut self, stmt: &Let<T>) -> T;
    fn visit_block(&mut self, stmt: &Block<T>) -> T;
    fn visit_if(&mut self, stmt: &If<T>) -> T;
    fn visit_while(&mut self, stmt: &While<T>) -> T;
    fn visit_fun(&mut self, stmt: &Fun<T>) -> T;
    fn visit_return(&mut self, stmt: &Return<T>) -> T;
    fn visit_class(&mut self, stmt: &Class<T>) -> T;
    fn visit_export(&mut self, stmt: &Export<T>) -> T;
    fn visit_use(&mut self, stmt: &Use<T>) -> T;
    fn visit_pipeline(&mut self, stmt: &Pipeline<T>) -> T;
    fn visit_buffer(&mut self, stmt: &Buffer<T>) -> T;
    fn visit_render(&mut self, render: &Render<T>) -> T;
}

pub trait CloneStmt<T> {
    fn clone_box(&self) -> Box<dyn Stmt<T>>;
}

pub trait Stmt<T>: Downcast + CloneStmt<T> + Send + Sync + 'static {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T;
}

impl_downcast!(Stmt<T>);

impl<T, R> CloneStmt<T> for R
where
    R: 'static + Stmt<T> + Clone,
    T: 'static + Clone,
{
    fn clone_box(&self) -> Box<dyn Stmt<T>> {
        Box::new(self.clone())
    }
}

impl<T> Clone for Box<dyn Stmt<T>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
