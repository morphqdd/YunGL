use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::environment::Environment;
use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::object::Object;
use crate::interpreter::scanner::token::token_type::TokenType;
use crate::interpreter::scanner::token::Token;
use crate::interpreter::Interpreter;
use crate::rc;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, RwLock};

type CallFn = Arc<dyn Fn(&mut Interpreter, Vec<Object>) -> Result<Object> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct Callable {
    id: u64,
    declaration: Option<Arc<RwLock<Fun<Result<Object>>>>>,
    closure: Option<Arc<RwLock<Environment>>>,
    call: CallFn,
    arity: Arc<dyn Fn() -> usize + Send + Sync + 'static>,
    to_string: Arc<dyn Fn() -> String + Send + Sync + 'static>,
    is_init: bool,
}

impl Callable {
    pub fn new(
        declaration: Option<Arc<RwLock<Fun<Result<Object>>>>>,
        closure: Option<Arc<RwLock<Environment>>>,
        is_init: bool,
    ) -> Self {
        let (id, name, params, body) = declaration.clone().unwrap().read().unwrap().clone().extract();
        let arity = params.len();
        let lexeme = name.get_lexeme().to_string();
        Self {
            id,
            declaration,
            closure: closure.clone(),
            is_init,
            call: rc!(move |interpreter, args| {
                let body = body.clone();
                let mut env = Environment::new(closure.clone());
                for i in 0..arity {
                    env.define(params[i].get_lexeme(), Some(args[i].clone()));
                }

                let closure = Arc::new(RwLock::new(env));

                match interpreter
                    .execute_block(body.iter().map(AsRef::as_ref).collect(), closure.clone())
                {
                    Ok(value) => {
                        if is_init {
                            return Environment::get_at(
                                Some(closure.clone()),
                                0,
                                &Token::new(
                                    TokenType::Identifier,
                                    "self",
                                    name.get_lit(),
                                    name.get_line(),
                                    name.get_pos_in_line(),
                                ),
                            );
                        }
                        Ok(value)
                    }
                    Err(err) => match err {
                        InterpreterError::Return(value) => {
                            if is_init {
                                return Environment::get_at(
                                    Some(closure.clone()),
                                    0,
                                    &Token::new(
                                        TokenType::Identifier,
                                        "self",
                                        name.get_lit(),
                                        name.get_line(),
                                        name.get_pos_in_line(),
                                    ),
                                );
                            }
                            Ok(value)
                        }
                        _ => Err(err),
                    },
                }
            }),
            arity: rc!(move || arity),
            to_string: rc!(move || lexeme.clone()),
        }
    }

    pub fn build(
        id: u64,
        declaration: Option<Arc<RwLock<Fun<Result<Object>>>>>,
        closure: Option<Arc<RwLock<Environment>>>,
        call: CallFn,
        arity: Arc<dyn Fn() -> usize + Send + Sync + 'static>,
        to_string: Arc<dyn Fn() -> String + Send + Sync + 'static>,
        is_init: bool,
    ) -> Self {
        Self {
            id,
            declaration,
            closure,
            call,
            arity,
            to_string,
            is_init,
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object> {
        (self.call)(interpreter, arguments)
    }

    pub fn arity(&self) -> usize {
        (self.arity)()
    }

    pub fn get_string(&self) -> String {
        (self.to_string)()
    }
    pub fn get_closure(&self) -> Option<Arc<RwLock<Environment>>> {
        self.closure.clone()
    }

    pub fn get_declaration(&self) -> Option<Arc<RwLock<Fun<Result<Object>>>>> {
        self.declaration.clone()
    }

    pub fn is_init(&self) -> bool {
        self.is_init
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{ <Callable {{ ... }}> }}", self.get_string())
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function#{} {}>", self.id, self.get_string())
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
