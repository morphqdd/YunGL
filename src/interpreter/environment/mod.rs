use crate::interpreter::error::Result;
use crate::interpreter::error::{RuntimeError, RuntimeErrorType};
use crate::interpreter::object::Object;
use crate::interpreter::scanner::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<Object>>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: Default::default(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Object>) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        if let Some(value) = self.values.get(name.get_lexeme()) {
            return match value {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError::new(
                    name.clone(),
                    RuntimeErrorType::VariableIsNotInit(name.get_lexeme().to_string()),
                )
                .into()),
            };
        }

        if let Some(enclosing) = self.enclosing.clone() {
            return enclosing.clone().borrow().get(name);
        }

        Err(RuntimeError::new(
            name.clone(),
            RuntimeErrorType::UndefinedVariable(name.get_lexeme().to_string()),
        )
        .into())
    }

    pub fn get_at(env: Option<Rc<RefCell<Self>>>, distance: usize, name: &Token) -> Result<Object> {
        if let Some(environment) = Environment::ancestor(env, distance) {
            return environment.borrow().get(name);
        }
        Err(RuntimeError::new(name.clone(), RuntimeErrorType::BugEnvironmentNotInit).into())
    }

    fn ancestor(
        env: Option<Rc<RefCell<Self>>>,
        distance: usize,
    ) -> Option<Rc<RefCell<Environment>>> {
        let mut env = env.clone();
        for _ in 0..distance {
            if let Some(env_) = env.clone() {
                env = env_.borrow().enclosing.clone();
            }
        }
        env
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<Object> {
        if self.values.contains_key(name.get_lexeme()) {
            self.values
                .insert(name.get_lexeme().to_string(), Some(value.clone()));
            return Ok(value);
        }

        if let Some(enclosing) = self.enclosing.clone() {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError::new(
            name.clone(),
            RuntimeErrorType::UndefinedVariable(name.get_lexeme().to_string()),
        )
        .into())
    }

    pub fn assign_at(
        env: Option<Rc<RefCell<Self>>>,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<Object> {
        if let Some(environment) = Environment::ancestor(env, distance) {
            environment
                .borrow_mut()
                .values
                .insert(name.get_lexeme().to_string(), Some(value.clone()));
            return Ok(value);
        }
        Err(RuntimeError::new(name.clone(), RuntimeErrorType::BugEnvironmentNotInit).into())
    }
    
    pub fn get_enclosing(&self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.clone()
    }
}
