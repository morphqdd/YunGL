use crate::interpreter::object::callable::Callable;
use crate::interpreter::object::instance::Instance;
use crate::interpreter::object::Object;
use crate::rc;
use crate::utils::next_id;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    id: u64,
    name: Rc<String>,
    methods: Rc<HashMap<String, Object>>,
    superclass: Option<Object>,
}

impl Class {
    pub fn new(
        name: String,
        methods: HashMap<String, Object>,
        superclass: Option<Object>,
    ) -> Self {
        Self {
            id: next_id(),
            name: rc!(name),
            methods: rc!(methods),
            superclass,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        if let Some(obj) = self.methods.get(name).cloned() {
            Some(obj)
        } else if let Some(superclass) = self.superclass.clone() {
            match superclass.inner() {
                Object::Class(class) => class.find_method(name),
                _ => panic!("Interpreter bug!")
            }
        } else {
            None
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<Class> for Callable {
    fn from(value: Class) -> Self {
        let name = value.name.clone();
        let value_call = value.clone();
        let value_arity = value.clone();
        let is_init = value.methods.get("init").is_some();
        Callable::build(
            value.id,
            None,
            None,
            rc!(move |interpreter, args| {
                let instance = Instance::new(value_call.clone());

                if let Some(initializer) = value_call.find_method("init") {
                    match initializer.bind(instance.clone())? {
                        Object::Callable(callable) => {
                            callable.call(interpreter, args)?;
                        }
                        _ => panic!("Interpreter bug!"),
                    }
                }

                Ok(Object::Instance(instance))
            }),
            rc!(
                move || if let Some(initializer) = value_arity.clone().find_method("init") {
                    match initializer {
                        Object::Callable(callable) => callable.arity(),
                        _ => panic!("Interpreter bug!"),
                    }
                } else {
                    0
                }
            ),
            rc!(move || name.to_string()),
            is_init,
        )
    }
}
