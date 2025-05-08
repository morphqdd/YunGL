use crate::interpreter::ast::expr::{Expr, ExprVisitor};
use crate::interpreter::ast::stmt::Stmt;
use crate::interpreter::scanner::token::Token;
use crate::utils::next_id;

#[derive(Clone)]
pub struct AnonFun<T: 'static> {
    id: u64,
    token: Token,
    params: Vec<Token>,
    body: Vec<Box<dyn Stmt<T>>>,
}

impl<T> AnonFun<T> {
    pub fn new(token: Token, params: Vec<Token>, body: Vec<Box<dyn Stmt<T>>>) -> Self {
        Self {
            id: next_id(),
            token,
            params,
            body,
        }
    }

    pub fn extract(&self) -> (u64, &Token, &[Token], Vec<&dyn Stmt<T>>) {
        (
            self.id,
            &self.token,
            self.params.as_ref(),
            self.body.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        )
    }
}

impl<T: 'static + Clone> Expr<T> for AnonFun<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        visitor.visit_anon(self)
    }

    fn id(&self) -> u64 {
        self.id
    }
}
