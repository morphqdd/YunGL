use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use crate::utils::next_id;

#[derive(Debug, Clone)]
pub struct NativeObject {
    id: u64,
    value: Box<dyn Native>,
}

impl NativeObject {
    pub fn new(value: Box<dyn Native>) -> Self {
        Self { id: next_id(), value }
    }

    pub fn extract(self) -> Box<dyn Native> {
        self.value
    }
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Native for Instant {
    fn clone_box(&self) -> Box<dyn Native> {
        Box::new(*self)
    }
}

pub trait Native: Debug + Downcast + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn Native>;
}

impl_downcast!(Native);

impl Clone for Box<dyn Native> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for NativeObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NativeObject {}

impl Hash for NativeObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}