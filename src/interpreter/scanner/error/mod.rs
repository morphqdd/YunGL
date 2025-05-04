use crate::interpreter::error::InterpreterError;
use crate::interpreter::Interpreter;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ScannerError {
    line: usize,
    pos_in_line: usize,
    ty: ScannerErrorType,
}

impl ScannerError {
    pub fn new(line: usize, pos_in_line: usize, ty: ScannerErrorType) -> Self {
        Self {
            line,
            pos_in_line,
            ty,
        }
    }
}

impl Display for ScannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Interpreter::error(self.line, self.pos_in_line, &self.ty.to_string())
        )
    }
}

impl From<ScannerError> for InterpreterError {
    fn from(value: ScannerError) -> Self {
        InterpreterError::ScannerError(value)
    }
}

#[derive(Debug, Clone)]
pub enum ScannerErrorType {
    UnexpectedCharacter(char),
    UnterminatedString,
}

impl Display for ScannerErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerErrorType::UnexpectedCharacter(ch) => write!(f, "Unexpected character '{ch}'"),
            ScannerErrorType::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

impl Error for ScannerErrorType {}
