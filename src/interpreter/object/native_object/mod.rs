use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct NativeObject {
    value: Box<dyn Native>,
}

impl NativeObject {
    pub fn new(value: Box<dyn Native>) -> Self {
        Self { value }
    }

    pub fn extract(self) -> Box<dyn Native> {
        self.value
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
