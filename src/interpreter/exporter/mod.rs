use crate::b;
use crate::interpreter::ast::expr::literal::Literal;
use crate::interpreter::ast::stmt::export_stmt::Export;
use crate::interpreter::ast::stmt::use_stmt::Use;
use crate::interpreter::ast::stmt::Stmt;
use crate::interpreter::error::Result;
use crate::interpreter::exporter::error::{ExporterError, ExporterErrorType};
use crate::interpreter::object::Object;
use crate::interpreter::parser::Parser;
use crate::interpreter::scanner::Scanner;
use std::fs::read_to_string;
use std::path::PathBuf;

pub mod error;

pub struct Exporter<T> {
    ast: Vec<Box<dyn Stmt<T>>>,
    path: PathBuf,
}

impl<T: 'static + Clone> Exporter<T> {
    pub fn new(path: PathBuf, ast: Vec<Box<dyn Stmt<T>>>) -> Self {
        Self { path, ast }
    }

    pub fn resolve(self) -> Result<Vec<Box<dyn Stmt<T>>>> {
        let mut ast = vec![];
        for stmt in self.ast.clone() {
            if let Some(use_stmt) = stmt.downcast_ref::<Use<T>>() {
                let (name, expr) = use_stmt.extract();
                if let Some(expr) = expr.downcast_ref::<Literal>() {
                    if let Some(Object::String(path)) = expr.get_value() {
                        let path = self.path.parent().unwrap().join(path.to_owned() + ".yun");
                        let code = read_to_string(path).unwrap();
                        let tokens = Scanner::new(&code).scan_tokens()?;
                        let exported_ast = self.sift(Parser::new(tokens).parse()?)?;
                        for stmt in exported_ast {
                            ast.push(stmt);
                        }
                    } else {
                        return Err(ExporterError::new(
                            name.clone(),
                            ExporterErrorType::ExpectedPathStringAfterUse,
                        )
                        .into());
                    }
                }
            } else {
                ast.push(stmt);
            }
        }
        Ok(ast)
    }

    pub fn sift(&self, ast: Vec<Box<dyn Stmt<T>>>) -> Result<Vec<Box<dyn Stmt<T>>>> {
        let mut sifted_ast: Vec<Box<dyn Stmt<T>>> = vec![];
        for stmt in ast {
            if let Some(export) = stmt.downcast_ref::<Export<T>>() {
                sifted_ast.push(b!(export.clone()));
            }
        }
        Ok(sifted_ast)
    }
}
