use crate::interpreter::ast::expr::variable::Variable;
use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::scanner::token::Token;

type ExtractedClass<'a, T> = (&'a Token, &'a Vec<Fun<T>>, Option<&'a Variable>);

#[derive(Clone)]
pub struct Class<T: 'static> {
    name: Token,
    #[allow(clippy::vec_box)]
    methods: Vec<Fun<T>>,
    super_class: Option<Variable>,
}

impl<T> Class<T> {
    pub fn new(name: Token, methods: Vec<Fun<T>>, super_class: Option<Variable>) -> Self {
        Self {
            name,
            methods,
            super_class,
        }
    }

    pub fn extract(&self) -> ExtractedClass<T> {
        (&self.name, &self.methods, self.super_class.as_ref())
    }
}

impl<T: 'static + Clone> Stmt<T> for Class<T> {
    fn accept(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        visitor.visit_class(self)
    }
}
