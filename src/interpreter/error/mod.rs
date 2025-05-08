use crate::interpreter::Interpreter;
use crate::interpreter::event::InterpreterEvent;
use crate::interpreter::exporter::error::ExporterError;
use crate::interpreter::object::Object;
use crate::interpreter::parser::error::ParserError;
use crate::interpreter::scanner::error::ScannerError;
use crate::interpreter::scanner::token::Token;
use glium::winit::error::EventLoopError;
use glium::winit::event_loop::EventLoopClosed;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, InterpreterError>;

#[derive(Debug, Clone, Error)]
pub enum InterpreterError {
    #[error("{0}")]
    ScannerError(ScannerError),
    #[error("{0}")]
    ParserError(ParserError),
    #[error("{0}")]
    ExporterError(ExporterError),
    #[error("{0}")]
    RuntimeError(RuntimeError),
    #[error("{0}")]
    RuntimeErrorType(RuntimeErrorType),
    #[error("{0}")]
    Custom(String),
    #[error("{0}")]
    Return(Object),
}

impl From<EventLoopError> for InterpreterError {
    fn from(value: EventLoopError) -> Self {
        InterpreterError::Custom(value.to_string())
    }
}

impl From<EventLoopClosed<InterpreterEvent>> for InterpreterError {
    fn from(value: EventLoopClosed<InterpreterEvent>) -> Self {
        InterpreterError::Custom(value.to_string())
    }
}

impl From<String> for InterpreterError {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<ParseFloatError> for InterpreterError {
    fn from(value: ParseFloatError) -> Self {
        println!("HERE");
        Self::Custom(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    token: Token,
    ty: RuntimeErrorType,
}

impl RuntimeError {
    pub fn new(token: Token, ty: RuntimeErrorType) -> Self {
        Self { token, ty }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Interpreter::error_by_token(self.token.clone(), &self.ty.to_string())
        )
    }
}

impl From<RuntimeError> for InterpreterError {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeErrorType {
    CannotAddTypes(String, String),
    CannotSubtractTypes(String, String),
    CannotMultiplyTypes(String, String),
    CannotDivideTypes(String, String),
    CannotNegateType(String),
    UnsupportedUnaryOperator(String),
    UnsupportedBinaryOperator(String),
    BugEnvironmentNotInit,
    UndefinedVariable(String),
    VariableIsNotInit(String),
    ArityOfFuncNotEqSizeOfArgs,
    NotCallable,
    UserPanicWithMsg(Object),
    CantToNum(String),
    OnlyInstancesHaveProperties,
    UndefinedProperty(String),
    SuperclassMustBeClass,
    MustBeANumber,
    OnlyListsHaveIndices,
}

impl Display for RuntimeErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeErrorType::CannotAddTypes(ty1, ty2) => {
                write!(f, "Cannot add types '{}' and '{}'", ty1, ty2)
            }
            RuntimeErrorType::CannotSubtractTypes(ty1, ty2) => {
                write!(f, "Cannot subtract '{}' and '{}'", ty1, ty2)
            }
            RuntimeErrorType::CannotMultiplyTypes(ty1, ty2) => {
                write!(f, "Cannot multiply '{}' and '{}'", ty1, ty2)
            }
            RuntimeErrorType::CannotDivideTypes(ty1, ty2) => {
                write!(f, "Cannot divide '{}' and '{}'", ty1, ty2)
            }
            RuntimeErrorType::CannotNegateType(ty) => write!(f, "Cannot negate '{}'", ty),
            RuntimeErrorType::UnsupportedUnaryOperator(op) => {
                write!(f, "Unsupported unary operator '{}'", op)
            }
            RuntimeErrorType::UnsupportedBinaryOperator(op) => {
                write!(f, "Unsupported binary operator '{}'", op)
            }
            RuntimeErrorType::BugEnvironmentNotInit => write!(f, "Bug environment not initialized"),
            RuntimeErrorType::UndefinedVariable(v) => write!(f, "Undefined variable '{}'", v),
            RuntimeErrorType::VariableIsNotInit(v) => {
                write!(f, "Variable '{}' is not initialized", v)
            }
            RuntimeErrorType::ArityOfFuncNotEqSizeOfArgs => {
                write!(f, "Arity of functions not equal size of args")
            }
            RuntimeErrorType::NotCallable => write!(f, "Not callable"),
            RuntimeErrorType::UserPanicWithMsg(msg) => write!(f, "{}", msg),
            RuntimeErrorType::CantToNum(ty) => {
                write!(f, "this type '{}' cannot be represented as a number", ty)
            }
            RuntimeErrorType::OnlyInstancesHaveProperties => {
                write!(f, "Only instances have properties")
            }
            RuntimeErrorType::UndefinedProperty(name) => write!(f, "Undefined property '{}'", name),
            RuntimeErrorType::SuperclassMustBeClass => write!(f, "Superclass must be class"),
            RuntimeErrorType::MustBeANumber => write!(f, "Must be a number"),
            RuntimeErrorType::OnlyListsHaveIndices => write!(f, "Only lists have indices"),
        }
    }
}

impl From<RuntimeErrorType> for InterpreterError {
    fn from(value: RuntimeErrorType) -> Self {
        Self::RuntimeErrorType(value)
    }
}
