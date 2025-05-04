use crate::interpreter::error::Result;
use crate::interpreter::error::{RuntimeError, RuntimeErrorType};
use crate::interpreter::object::class::Class;
use crate::interpreter::object::Object;
use crate::interpreter::scanner::token::Token;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Instance {
    class: Arc<Class>,
    fields: Arc<RwLock<HashMap<String, Object>>>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class: Arc::new(class),
            fields: Default::default(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        if let Some(obj) = self.fields.read().unwrap().get(name.get_lexeme()) {
            return Ok(obj.clone());
        }

        if let Some(method) = self.class.find_method(name.get_lexeme()) {
            return method.bind(self.clone());
        }

        Err(RuntimeError::new(
            name.clone(),
            RuntimeErrorType::UndefinedProperty(name.get_lexeme().to_string()),
        )
        .into())
    }

    pub fn set(&self, name: &Token, value: Object) {
        self.fields
            .write().unwrap()
            .insert(name.get_lexeme().to_string(), value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class && self.fields.read().unwrap().eq(&other.fields.read().unwrap())
    }
}
