use std::sync::atomic::AtomicU64;

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[macro_export]
macro_rules! b {
    ($e: expr) => {
        Box::new($e)
    };
}

#[macro_export]
macro_rules! rc {
    ($e: expr) => {
        Rc::new($e)
    };
}
