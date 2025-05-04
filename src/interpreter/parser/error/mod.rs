use crate::interpreter::error::InterpreterError;
use crate::interpreter::scanner::token::Token;
use crate::interpreter::Interpreter;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ParserError {
    token: Token,
    ty: ParserErrorType,
}

impl ParserError {
    pub fn new(token: Token, ty: ParserErrorType) -> Self {
        Self { token, ty }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Interpreter::error_by_token(self.token.clone(), &self.ty.to_string())
        )
    }
}

impl From<ParserError> for InterpreterError {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}
#[derive(Debug, Clone)]
pub enum ParserErrorType {
    ExpectedMatchingParens,
    ExpectedExpression,
    ExpectedSemicolon,
    ExpectedVariableName,
    ExpectedSemicolonAfterVarDecl,
    UndefinedVariable(String),
    InvalidAssignmentTarget,
    ExpectedMatchingBrace,
    ExpectedLeftParenAfterIf,
    ExpectedRightParenAfterIfCondition,
    ExpectedRightParenAfterForStatement,
    ExpectedLeftParenAfterFor,
    ExpectedLeftParenAfterWhile,
    ExpectedRightParenAfterWhileStatement,
    ExpectedRightParenAfterArguments,
    CountOfArgsGreaterThen255,
    ExpectedIdentAfterFunDecl,
    ExpectedLeftParenAfterFunIdent,
    CountOfParamsGreaterThen255,
    ExpectedRightParenAfterParams,
    ExpectedLeftBraceBeforeBody,
    ExpectedParamName,
    CantReadLocalVariableInItsOwnInit,
    CantReturnFromTopLevelCode,
    ExpectedIdentAfterClassDecl,
    NotAFunc,
    ExpectedPropertyAfterDot,
    CantUseSelfOutsideClass,
    CantReturnFromInitializer,
    ExpectedSuperClassIdent,
    CantInheritItSelf,
    ExpectedMethodAfterDot,
    ExpectedDotAfterSuper,
    CantUseSuperOutsideOfClass,
    CantUseSuperInClassWithoutSuperClasses,
    ExpectedRightBracket,
    ExpectedRightBrace,
    ExpectedColon,
    ExpectedKey,
}

impl Display for ParserErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorType::ExpectedMatchingParens => write!(f, "Expect ')', after expression!"),
            ParserErrorType::ExpectedExpression => write!(f, "Expected expression!"),
            ParserErrorType::ExpectedSemicolon => write!(f, "Expected ';' after expression!"),
            ParserErrorType::ExpectedVariableName => write!(f, "Expected variable name!"),
            ParserErrorType::ExpectedSemicolonAfterVarDecl => {
                write!(f, "Expected ';' after variable declaration!")
            }
            ParserErrorType::UndefinedVariable(v) => write!(f, "Undefined variable: `{}`!", v),
            ParserErrorType::InvalidAssignmentTarget => write!(f, "Invalid assignment target!"),
            ParserErrorType::ExpectedMatchingBrace => write!(f, "Expected '}}' after block!"),
            ParserErrorType::ExpectedLeftParenAfterIf => write!(f, "Expected '(' after if!"),
            ParserErrorType::ExpectedRightParenAfterIfCondition => {
                write!(f, "Expected ')' after if condition!")
            }
            ParserErrorType::ExpectedRightParenAfterForStatement => {
                write!(f, "Expected ')' after for statement!")
            }
            ParserErrorType::ExpectedLeftParenAfterFor => write!(f, "Expected '( after for!"),
            ParserErrorType::ExpectedLeftParenAfterWhile => write!(f, "Expected '( after while!"),
            ParserErrorType::ExpectedRightParenAfterWhileStatement => {
                write!(f, "Expected ')' after while statement!")
            }
            ParserErrorType::ExpectedRightParenAfterArguments => {
                write!(f, "Expected ')' after arguments!")
            }
            ParserErrorType::CountOfArgsGreaterThen255 => {
                write!(f, "Number of arguments greater than 255!")
            }
            ParserErrorType::ExpectedIdentAfterFunDecl => {
                write!(f, "Expected identifier after function declaration!")
            }
            ParserErrorType::ExpectedLeftParenAfterFunIdent => {
                write!(f, "Expected '(' after function identifier!")
            }
            ParserErrorType::CountOfParamsGreaterThen255 => {
                write!(f, "Number of params greater than 255!")
            }
            ParserErrorType::ExpectedRightParenAfterParams => {
                write!(f, "Expected ')' after parameters!")
            }
            ParserErrorType::ExpectedLeftBraceBeforeBody => write!(f, "Expected '{{' after body!"),
            ParserErrorType::ExpectedParamName => write!(f, "Expected parameter name!"),
            ParserErrorType::CantReadLocalVariableInItsOwnInit => {
                write!(f, "Can't read local variable in it's own init!")
            }
            ParserErrorType::CantReturnFromTopLevelCode => {
                write!(f, "Can't return from top level code!")
            }
            ParserErrorType::ExpectedIdentAfterClassDecl => {
                write!(f, "Expected identifier after class declaration!")
            }
            ParserErrorType::NotAFunc => write!(f, "Not a function!"),
            ParserErrorType::ExpectedPropertyAfterDot => write!(f, "Expected property after '.'!"),
            ParserErrorType::CantUseSelfOutsideClass => {
                write!(f, "Can't use 'self' outside of a class!")
            }
            ParserErrorType::CantReturnFromInitializer => {
                write!(f, "Can't return from initializer!")
            }
            ParserErrorType::ExpectedSuperClassIdent => {
                write!(f, "Expected superclass identifier!")
            }
            ParserErrorType::CantInheritItSelf => write!(f, "A class can't inherit from itself!"),
            ParserErrorType::ExpectedMethodAfterDot => write!(f, "Expected method after '.'!"),
            ParserErrorType::ExpectedDotAfterSuper => write!(f, "Expected '.'!"),
            ParserErrorType::CantUseSuperOutsideOfClass => write!(f, "Can't use 'super' outside of a class!"),
            ParserErrorType::CantUseSuperInClassWithoutSuperClasses => write!(f, "Can't use 'super' in class without superclasses!"),
            ParserErrorType::ExpectedRightBracket => write!(f, "Expected ']'!"),
            ParserErrorType::ExpectedRightBrace => write!(f, "Expected '}}'!"),
            ParserErrorType::ExpectedColon => write!(f,"Expected ':'!"),
            ParserErrorType::ExpectedKey => write!(f, "Expected key!"),
        }
    }
}
