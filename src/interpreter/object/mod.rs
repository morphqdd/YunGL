use crate::b;
use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::environment::Environment;
use crate::interpreter::error::RuntimeErrorType;
use crate::interpreter::error::{InterpreterError, Result};
use crate::interpreter::object::callable::Callable;
use crate::interpreter::object::class::Class;
use crate::interpreter::object::instance::Instance;
use crate::interpreter::object::native_object::NativeObject;
use crate::interpreter::parser::resolver::SomeFun;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Deref, Div, Mul, Neg, Not, Sub};
use std::sync::{Arc, RwLock};

pub mod callable;
pub mod class;
pub mod instance;
pub mod native_object;

#[derive(Debug, Clone, Default)]
pub enum Object {
    String(String),
    Number(f64),
    Bool(bool),
    Callable(Callable),
    Class(Box<Class>),
    Instance(Instance),
    NativeObject(NativeObject),
    Arc(Arc<Object>),
    #[default]
    Nil,
    Void,
    List(Arc<RwLock<Vec<Object>>>),
    Dictionary(Arc<RwLock<HashMap<String, Object>>>),
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Object::*;
        std::mem::discriminant(self).hash(state);
        match self {
            String(s) => s.hash(state),
            Number(n) => OrderedFloat(*n).hash(state),
            Bool(b) => b.hash(state),
            Callable(c) => c.hash(state),
            Class(c) => c.hash(state),
            Instance(i) => i.hash(state),
            NativeObject(n) => n.hash(state),
            Arc(o) => o.hash(state),
            Nil => {} // ничего не нужно
            Void => {}
            List(vec) => vec.read().unwrap().hash(state),
            Dictionary(map) => {
                for (k, v) in map.read().unwrap().iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl Object {
    pub fn get_type(&self) -> String {
        match self {
            Object::String(_) => "string".into(),
            Object::Number(_) => "number".into(),
            Object::Bool(_) => "boolean".into(),
            Object::Nil => "nil".into(),
            Object::Void => "void".into(),
            Object::Callable { .. } => "<callable>".into(),
            Object::Class(class) => class.to_string(),
            Object::Instance(instance) => instance.to_string(),
            Object::NativeObject(_) => "<native object>".into(),
            Object::Arc(obj) => obj.get_type(),
            Object::List(_) => "list".into(),
            Object::Dictionary(_) => "dictionary".into(),
        }
    }

    pub fn function(
        stmt: SomeFun,
        closure: Option<Arc<RwLock<Environment>>>,
        is_init: bool,
    ) -> Self {
        Self::Callable(Callable::new(
            Some(Arc::new(RwLock::new(stmt))),
            closure.clone(),
            is_init,
        ))
    }

    pub fn class(name: &str, methods: HashMap<String, Object>, superclass: Option<Object>) -> Self {
        Self::Class(b!(Class::new(name.to_string(), methods, superclass)))
    }

    pub fn bind(&self, obj: Instance) -> Result<Object> {
        match self {
            Object::Callable(callable) => {
                let mut env = Environment::new(callable.get_closure());
                env.define("self", Some(Object::Instance(obj)));
                Ok(Object::Callable(Callable::new(
                    callable.get_declaration(),
                    Some(Arc::new(RwLock::new(env))),
                    callable.is_init(),
                )))
            }
            _ => panic!("Interpreter bug"),
        }
    }

    pub fn clone_into_rc(&self) -> Self {
        match self {
            Object::Arc(obj) => obj.clone().deref().clone(),
            _ => self.clone(),
        }
    }

    pub fn inner(&self) -> &Self {
        match self {
            Object::Arc(rc) => rc.inner(),
            _ => self,
        }
    }

    pub fn get_field(&self, key: Object) -> Option<Object> {
        if let Object::Dictionary(map) = self {
            let Object::String(key) = key else {
                return None;
            };
            return Some(
                map.read()
                    .unwrap()
                    .get(&key)
                    .cloned()
                    .unwrap_or(Object::Nil),
            );
        }
        if let Object::List(list) = self {
            let Object::Number(i) = key else { return None };
            return Some(
                list.read()
                    .unwrap()
                    .get(i as usize)
                    .unwrap_or(&Object::Nil)
                    .clone(),
            );
        }
        None
    }
}

impl Neg for Object {
    type Output = Result<Object>;

    fn neg(self) -> Self::Output {
        match self {
            Object::Number(n) => Ok(Object::Number(-n)),
            Object::Arc(rc) => -rc.clone_into_rc(),
            _ => Err(RuntimeErrorType::CannotNegateType(self.get_type()).into()),
        }
    }
}

impl Not for Object {
    type Output = Result<Object>;

    fn not(self) -> Self::Output {
        match self {
            Object::Bool(b) => Ok(Object::Bool(!b)),
            Object::Nil => Ok(Object::Bool(true)),
            Object::Void => Ok(Object::Bool(true)),
            _ => Ok(Object::Bool(false)),
        }
    }
}

impl Not for &Object {
    type Output = Result<Object>;

    fn not(self) -> Self::Output {
        match self {
            Object::Bool(b) => Ok(Object::Bool(!b)),
            Object::Nil => Ok(Object::Bool(true)),
            Object::Void => Ok(Object::Bool(true)),
            _ => Ok(Object::Bool(false)),
        }
    }
}

impl Add for Object {
    type Output = Result<Object>;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Object::Number(a), Object::Number(b)) => Ok(Object::Number(a + b)),
            (Object::String(a), Object::String(b)) => Ok(Object::String(a.to_owned() + b)),
            (Object::List(a), _) => {
                a.write().unwrap().push(rhs.clone());
                Ok(Object::List(a.clone()))
            }
            (Object::Arc(rc), _) => rc.clone_into_rc() + rhs,
            (_, Object::Arc(rc)) => self + rc.clone_into_rc(),
            _ => Err(RuntimeErrorType::CannotAddTypes(self.get_type(), rhs.get_type()).into()),
        }
    }
}

impl Sub for Object {
    type Output = Result<Object>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Object::Number(a), Object::Number(b)) => Ok(Object::Number(a - b)),
            (Object::Arc(rc), _) => rc.clone_into_rc() - rhs,
            (_, Object::Arc(rc)) => self - rc.clone_into_rc(),
            _ => Err(RuntimeErrorType::CannotSubtractTypes(self.get_type(), rhs.get_type()).into()),
        }
    }
}

impl Mul for Object {
    type Output = Result<Object>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Object::Number(a), Object::Number(b)) => Ok(Object::Number(a * b)),
            (Object::Arc(rc), _) => rc.clone_into_rc() * rhs,
            (_, Object::Arc(rc)) => self * rc.clone_into_rc(),
            _ => Err(RuntimeErrorType::CannotMultiplyTypes(self.get_type(), rhs.get_type()).into()),
        }
    }
}

impl Div for Object {
    type Output = Result<Object>;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Object::Number(a), Object::Number(b)) => Ok(Object::Number(a / b)),
            (Object::Arc(rc), _) => rc.clone_into_rc() / rhs,
            (_, Object::Arc(rc)) => self / rc.clone_into_rc(),
            _ => Err(RuntimeErrorType::CannotDivideTypes(self.get_type(), rhs.get_type()).into()),
        }
    }
}

impl PartialEq<Self> for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Bool(a), Object::Bool(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            (Object::Void, Object::Void) => true,
            (Object::Callable(callable), Object::Callable(callable2)) => callable == callable2,
            (Object::Arc(rc), _) => &rc.clone_into_rc() == other,
            (_, Object::Arc(rc)) => self == &rc.clone_into_rc(),
            (Object::Dictionary(dict), Object::Dictionary(dict2)) => {
                *dict.read().unwrap() == *dict2.read().unwrap()
            }
            (Object::List(dict), Object::List(dict2)) => {
                *dict.read().unwrap() == *dict2.read().unwrap()
            }
            _ => false,
        }
    }
}

impl PartialOrd<Self> for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Number(a), Object::Number(b)) => a.partial_cmp(b),
            (Object::String(a), Object::String(b)) => a.partial_cmp(b),
            (Object::Bool(a), Object::Bool(b)) => a.partial_cmp(b),
            (Object::Arc(rc), _) => rc.clone_into_rc().partial_cmp(other),
            (_, Object::Arc(rc)) => self.partial_cmp(&rc.clone_into_rc()),
            (Object::Nil, Object::Nil) => None,
            _ => Some(Ordering::Equal),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(str) => write!(f, "{}", str),
            Object::Number(num) => write!(f, "{}", num),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::Void => write!(f, ""),
            Object::Callable(callable) => write!(f, "{}", callable),
            Object::Class(class) => write!(f, "{}", class),
            Object::Instance(instance) => write!(f, "{}", instance),
            Object::NativeObject(_) => write!(f, "<native object>"),
            Object::Arc(rc) => write!(f, "{}", rc),
            Object::List(list) => write!(
                f,
                "[{}]",
                list.read()
                    .unwrap()
                    .iter()
                    .map(|obj| match obj {
                        Object::String(str) => format!("{:?}", str),
                        _ => obj.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Object::Dictionary(obj) => write!(
                f,
                "{{{}}}",
                obj.read()
                    .unwrap()
                    .iter()
                    .map(|(key, obj)| match obj {
                        Object::String(str) => format!("{}: {:?}", key, str),
                        _ => format!("{}: {}", key, obj),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl From<Object> for Result<i32> {
    fn from(value: Object) -> Self {
        match value {
            Object::Number(n) => Ok(n as i32),
            _ => Err(RuntimeErrorType::CantToNum(value.get_type()).into()),
        }
    }
}

impl From<Object> for InterpreterError {
    fn from(value: Object) -> Self {
        InterpreterError::Return(value)
    }
}
