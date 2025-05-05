use crate::interpreter::ast::expr::assignment::Assign;
use crate::interpreter::ast::expr::binary::Binary;
use crate::interpreter::ast::expr::call::Call;
use crate::interpreter::ast::expr::get::Get;
use crate::interpreter::ast::expr::grouping::Grouping;
use crate::interpreter::ast::expr::literal::Literal;
use crate::interpreter::ast::expr::logical::Logical;
use crate::interpreter::ast::expr::self_expr::SelfExpr;
use crate::interpreter::ast::expr::set::Set;
use crate::interpreter::ast::expr::superclass::Super;
use crate::interpreter::ast::expr::unary::Unary;
use crate::interpreter::ast::expr::variable::Variable;
use crate::interpreter::ast::expr::{Expr, ExprVisitor};
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
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::error::Result;
use crate::interpreter::object::Object;
use crate::interpreter::parser::error::{ParserError, ParserErrorType};
use crate::interpreter::scanner::token::Token;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::ops::Deref;
use crate::interpreter::ast::expr::list::List;
use crate::interpreter::ast::expr::object::Obj;
use crate::interpreter::ast::stmt::buffer::Buffer;
use crate::interpreter::ast::stmt::pipeline::Pipeline;
use crate::interpreter::ast::stmt::render::Render;

#[derive(Clone, Copy, PartialEq)]
pub enum FunctionType {
    Function,
    Method,
    Initializer,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClassType {
    Class,
    SubClass,
    None,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    stack: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a>
where
    Resolver<'a>: ExprVisitor<Result<Object>> + StmtVisitor<Result<Object>>,
{
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            stack: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, stmts: Vec<&dyn Stmt<Result<Object>>>) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &dyn Stmt<Result<Object>>) -> Result<Object> {
        stmt.accept(self)
    }

    fn resolve_expr(&mut self, expr: &dyn Expr<Result<Object>>) -> Result<Object> {
        expr.accept(self)
    }

    fn begin_scope(&mut self) {
        self.stack.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.stack.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.stack.is_empty() {
            return;
        }

        let scope = self.stack.last_mut().unwrap();
        scope.insert(name.get_lexeme().to_string(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.stack.is_empty() {
            return;
        }

        let scope = self.stack.last_mut().unwrap();
        scope.insert(name.get_lexeme().to_string(), true);
    }

    fn resolve_local(&mut self, expr: &dyn Expr<Result<Object>>, name: &Token) {
        if self.stack.is_empty() {
            return;
        }

        for i in (0..=self.stack.len().saturating_sub(1)).rev() {
            if self.stack.get(i).unwrap().contains_key(name.get_lexeme()) {
                self.interpreter
                    .resolve(expr.clone_expr(), self.stack.len() - i - 1);
                return;
            }
        }
    }

    fn resolve_function(&mut self, func: &Fun<Result<Object>>, ty: FunctionType) -> Result<()> {
        let enclosing_func = self.current_function;
        self.current_function = ty;

        self.begin_scope();
        let (_, _, params, body) = func.clone().extract();
        for param in params {
            self.declare(&param);
            self.define(&param);
        }
        self.resolve(body.iter().map(AsRef::as_ref).collect::<Vec<_>>())?;
        self.end_scope();

        self.current_function = enclosing_func;
        Ok(())
    }
}

impl ExprVisitor<Result<Object>> for Resolver<'_> {
    fn visit_binary(&mut self, binary: &Binary<Result<Object>>) -> Result<Object> {
        self.resolve_expr(binary.get_left())?;
        self.resolve_expr(binary.get_right())?;
        Ok(Object::Nil)
    }

    fn visit_grouping(&mut self, grouping: &Grouping<Result<Object>>) -> Result<Object> {
        self.resolve_expr(grouping.get_expr())?;
        Ok(Object::Nil)
    }

    fn visit_literal(&mut self, _literal: &Literal) -> Result<Object> {
        Ok(Object::Nil)
    }

    fn visit_unary(&mut self, unary: &Unary<Result<Object>>) -> Result<Object> {
        self.resolve_expr(unary.get_right())?;
        Ok(Object::Nil)
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result<Object> {
        let name = variable.get_token();

        if let Some(scope) = self.stack.last() {
            if let Some(value) = scope.get(name.get_lexeme()) {
                if !(*value) {
                    return Err(ParserError::new(
                        name,
                        ParserErrorType::CantReadLocalVariableInItsOwnInit,
                    )
                    .into());
                }
            }
        }

        self.resolve_local(variable, &name);
        Ok(Object::Nil)
    }

    fn visit_assign(&mut self, assign: &Assign<Result<Object>>) -> Result<Object> {
        self.resolve_expr(assign.get_value())?;
        self.resolve_local(assign, &assign.get_token());
        Ok(Object::Nil)
    }

    fn visit_logical(&mut self, logical: &Logical<Result<Object>>) -> Result<Object> {
        self.resolve_expr(logical.get_left())?;
        self.resolve_expr(logical.get_right())?;
        Ok(Object::Nil)
    }

    fn visit_call(&mut self, call: &Call<Result<Object>>) -> Result<Object> {
        self.resolve_expr(call.get_callable())?;
        for arg in call.get_args() {
            self.resolve_expr(arg)?;
        }
        Ok(Object::Nil)
    }

    fn visit_get(&mut self, get: &Get<Result<Object>>) -> Result<Object> {
        self.resolve_expr(get.extract().1)?;
        Ok(Object::Nil)
    }

    fn visit_set(&mut self, set: &Set<Result<Object>>) -> Result<Object> {
        let (_, obj, value) = set.extract();
        self.resolve_expr(obj)?;
        self.resolve_expr(value)?;
        Ok(Object::Nil)
    }

    fn visit_self(&mut self, self_val: &SelfExpr) -> Result<Object> {
        if self.current_class == ClassType::None {
            return Err(ParserError::new(
                self_val.get_name(),
                ParserErrorType::CantUseSelfOutsideClass,
            )
            .into());
        }
        self.resolve_local(self_val, &self_val.get_name());
        Ok(Object::Nil)
    }

    fn visit_super(&mut self, super_val: &Super) -> Result<Object> {
        if self.current_class == ClassType::None {
            return Err(ParserError::new(super_val.extract().0, ParserErrorType::CantUseSuperOutsideOfClass).into())
        } else if self.current_class != ClassType::SubClass {
            return Err(ParserError::new(super_val.extract().0, ParserErrorType::CantUseSuperInClassWithoutSuperClasses).into())
        }
        self.resolve_local(super_val, &super_val.extract().0);
        Ok(Object::Nil)
    }

    fn visit_list(&mut self, list: &List<Result<Object>>) -> Result<Object> {
        let values = list.extract_values();
        for value in values {
            self.resolve_expr(value)?;
        }
        Ok(Object::Nil)
    }

    fn visit_object(&mut self, object: &Obj<Result<Object>>) -> Result<Object> {
        let values = object.extract();
        self.begin_scope();
        for (key, value) in values.clone() {
            self.declare(&key);
            self.resolve_expr(value.deref())?;
            self.define(&key);
        }
        self.end_scope();
        Ok(Object::Nil)
    }
}

impl StmtVisitor<Result<Object>> for Resolver<'_> {
    fn visit_expr(&mut self, stmt: &StmtExpr<Result<Object>>) -> Result<Object> {
        self.resolve_expr(stmt.expr())?;
        Ok(Object::Nil)
    }

    fn visit_print(&mut self, stmt: &Print<Result<Object>>) -> Result<Object> {
        self.resolve_expr(stmt.expr())?;
        Ok(Object::Nil)
    }

    fn visit_let(&mut self, stmt: &Let<Result<Object>>) -> Result<Object> {
        let name = stmt.get_ident();
        self.declare(&name);
        if let Some(initializer) = stmt.get_initializer() {
            self.resolve_expr(initializer)?;
        }
        self.define(&name);
        Ok(Object::Nil)
    }

    fn visit_block(&mut self, stmt: &Block<Result<Object>>) -> Result<Object> {
        self.begin_scope();
        self.resolve(stmt.get_stmts())?;
        self.end_scope();
        Ok(Object::Nil)
    }

    fn visit_if(&mut self, stmt: &If<Result<Object>>) -> Result<Object> {
        let (cond, then, else_) = stmt.extract();
        self.resolve_expr(cond)?;
        self.resolve_stmt(then)?;
        if let Some(else_) = else_ {
            self.resolve_stmt(else_)?;
        }
        Ok(Object::Nil)
    }

    fn visit_while(&mut self, stmt: &While<Result<Object>>) -> Result<Object> {
        let (cond, body) = stmt.extract();
        self.resolve_expr(cond)?;
        self.resolve_stmt(body)?;
        Ok(Object::Nil)
    }

    fn visit_fun(&mut self, stmt: &Fun<Result<Object>>) -> Result<Object> {
        let name = stmt.get_name();
        self.declare(&name);
        self.define(&name);
        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(Object::Nil)
    }

    fn visit_return(&mut self, stmt: &Return<Result<Object>>) -> Result<Object> {
        let (name, expr) = stmt.extract();

        if self.current_function == FunctionType::None {
            return Err(ParserError::new(
                name.clone(),
                ParserErrorType::CantReturnFromTopLevelCode,
            )
            .into());
        }

        if let Some(expr) = expr {
            if self.current_function == FunctionType::Initializer {
                return Err(ParserError::new(
                    name.clone(),
                    ParserErrorType::CantReturnFromInitializer,
                )
                .into());
            }
            self.resolve_expr(expr)?;
        }
        Ok(Object::Nil)
    }

    fn visit_class(&mut self, class: &Class<Result<Object>>) -> Result<Object> {
        let (name, methods, super_class) = class.extract();
        let enclosing_ty = self.current_class;
        self.current_class = ClassType::Class;

        self.define(name);
        self.declare(name);

        if let Some(super_class) = super_class {
            let s_name = super_class.get_token();
            if s_name.get_lexeme().eq(name.get_lexeme()) {
                return Err(ParserError::new(s_name, ParserErrorType::CantInheritItSelf).into());
            }

            self.current_class = ClassType::SubClass;
            self.resolve_expr(super_class)?;

            self.begin_scope();
            self.stack.last_mut().unwrap().insert("super".into(), true);
        }

        self.begin_scope();

        self.stack.last_mut().unwrap().insert("self".into(), true);

        for method in methods {
            let mut ty = FunctionType::Method;

            if method.get_name().get_lexeme().eq("init") {
                ty = FunctionType::Initializer;
            }

            self.resolve_function(&method.clone(), ty)?
        }

        self.end_scope();

        if super_class.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_ty;

        Ok(Object::Nil)
    }

    fn visit_export(&mut self, class: &Export<Result<Object>>) -> Result<Object> {
        let (_, stmt) = class.extract();
        self.resolve_stmt(stmt)
    }

    fn visit_use(&mut self, _stmt: &Use<Result<Object>>) -> Result<Object> {
        let (_, expr) = _stmt.extract();
        self.resolve_expr(expr)
    }

    fn visit_pipeline(&mut self, stmt: &Pipeline<Result<Object>>) -> Result<Object> {
        let (name, expr) = stmt.extract();
        self.declare(&name);
        self.define(&name);
        self.resolve_expr(expr)
    }

    fn visit_buffer(&mut self, stmt: &Buffer<Result<Object>>) -> Result<Object> {
        let (name, expr) = stmt.extract();
        self.declare(&name);
        self.define(&name);
        self.resolve_expr(expr)
    }

    fn visit_render(&mut self, render: &Render<Result<Object>>) -> Result<Object> {
        let exprs = render.extract();
        for expr in exprs {
            self.resolve_expr(expr)?;
        }
        Ok(Object::Nil)
    }
}
