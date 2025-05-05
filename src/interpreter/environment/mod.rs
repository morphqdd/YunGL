use crate::interpreter::error::Result;
use crate::interpreter::error::{RuntimeError, RuntimeErrorType};
use crate::interpreter::object::Object;
use crate::interpreter::scanner::token::Token;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<Object>>,
    enclosing: Option<Arc<RwLock<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Environment {
    pub fn new(enclosing: Option<Arc<RwLock<Environment>>>) -> Self {
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
            return enclosing.clone().read().unwrap().get(name);
        }

        Err(RuntimeError::new(
            name.clone(),
            RuntimeErrorType::UndefinedVariable(name.get_lexeme().to_string()),
        )
        .into())
    }

    pub fn get_at(env: Option<Arc<RwLock<Self>>>, distance: usize, name: &Token) -> Result<Object> {
        if let Some(environment) = Environment::ancestor(env, distance) {
            return environment.read().unwrap().get(name);
        }
        Err(RuntimeError::new(name.clone(), RuntimeErrorType::BugEnvironmentNotInit).into())
    }

    fn ancestor(
        env: Option<Arc<RwLock<Self>>>,
        distance: usize,
    ) -> Option<Arc<RwLock<Environment>>> {
        let mut env = env.clone();
        for _ in 0..distance {
            if let Some(env_) = env.clone() {
                env = env_.read().unwrap().enclosing.clone();
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
            return enclosing.write().unwrap().assign(name, value);
        }

        Err(RuntimeError::new(
            name.clone(),
            RuntimeErrorType::UndefinedVariable(name.get_lexeme().to_string()),
        )
        .into())
    }

    pub fn assign_at(
        env: Option<Arc<RwLock<Self>>>,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<Object> {
        if let Some(environment) = Environment::ancestor(env, distance) {
            environment
                .write()
                .unwrap()
                .values
                .insert(name.get_lexeme().to_string(), Some(value.clone()));
            return Ok(value);
        }
        Err(RuntimeError::new(name.clone(), RuntimeErrorType::BugEnvironmentNotInit).into())
    }

    pub fn get_enclosing(&self) -> Option<Arc<RwLock<Environment>>> {
        self.enclosing.clone()
    }
}
