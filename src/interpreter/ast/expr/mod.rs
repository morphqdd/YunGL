use crate::interpreter::ast::expr::assignment::Assign;
use crate::interpreter::ast::expr::binary::Binary;
use crate::interpreter::ast::expr::call::Call;
use crate::interpreter::ast::expr::get::Get;
use crate::interpreter::ast::expr::grouping::Grouping;
use crate::interpreter::ast::expr::list::List;
use crate::interpreter::ast::expr::literal::Literal;
use crate::interpreter::ast::expr::logical::Logical;
use crate::interpreter::ast::expr::object::Obj;
use crate::interpreter::ast::expr::self_expr::SelfExpr;
use crate::interpreter::ast::expr::set::Set;
use crate::interpreter::ast::expr::superclass::Super;
use crate::interpreter::ast::expr::unary::Unary;
use crate::interpreter::ast::expr::variable::Variable;
use downcast_rs::{Downcast, impl_downcast};

pub mod assignment;
pub mod binary;
pub mod call;
pub mod get;
pub mod grouping;
pub mod list;
pub mod literal;
pub mod logical;
pub mod object;
pub mod self_expr;
pub mod set;
pub mod superclass;
pub mod unary;
pub mod variable;

pub trait ExprVisitor<T> {
    fn visit_binary(&mut self, binary: &Binary<T>) -> T;
    fn visit_grouping(&mut self, grouping: &Grouping<T>) -> T;
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_unary(&mut self, unary: &Unary<T>) -> T;
    fn visit_variable(&mut self, variable: &Variable) -> T;
    fn visit_assign(&mut self, assign: &Assign<T>) -> T;
    fn visit_logical(&mut self, logical: &Logical<T>) -> T;
    fn visit_call(&mut self, call: &Call<T>) -> T;
    fn visit_get(&mut self, get: &Get<T>) -> T;
    fn visit_set(&mut self, set: &Set<T>) -> T;
    fn visit_self(&mut self, self_val: &SelfExpr) -> T;
    fn visit_super(&mut self, super_val: &Super) -> T;
    fn visit_list(&mut self, list: &List<T>) -> T;
    fn visit_object(&mut self, object: &Obj<T>) -> T;
}

pub trait Expr<T>: Downcast + CloneExpr<T> + Send + Sync + 'static {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T;
    fn id(&self) -> u64;
}

impl_downcast!(Expr<T>);

pub trait CloneExpr<T> {
    fn clone_expr(&self) -> Box<dyn Expr<T>>;
}

impl<T, R> CloneExpr<T> for R
where
    R: 'static + Expr<T> + Clone,
{
    fn clone_expr(&self) -> Box<dyn Expr<T>> {
        Box::new(self.clone())
    }
}

impl<T: 'static + Clone> Clone for Box<dyn Expr<T>> {
    fn clone(&self) -> Box<dyn Expr<T>> {
        self.clone_expr()
    }
}
