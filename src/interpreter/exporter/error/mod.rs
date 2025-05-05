use crate::interpreter::Interpreter;
use crate::interpreter::error::InterpreterError;
use crate::interpreter::scanner::token::Token;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ExporterError {
    token: Token,
    ty: ExporterErrorType,
}

impl ExporterError {
    pub fn new(token: Token, ty: ExporterErrorType) -> Self {
        Self { token, ty }
    }
}

impl Display for ExporterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Interpreter::error_by_token(self.token.clone(), &self.ty.to_string())
        )
    }
}

impl From<ExporterError> for InterpreterError {
    fn from(value: ExporterError) -> Self {
        InterpreterError::ExporterError(value)
    }
}

#[derive(Debug, Clone)]
pub enum ExporterErrorType {
    ExpectedPathStringAfterUse,
}

impl Display for ExporterErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExporterErrorType::ExpectedPathStringAfterUse => {
                write!(f, "Expected path string after use")
            }
        }
    }
}
