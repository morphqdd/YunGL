use cgmath::Matrix4;
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
        Arc::new($e)
    };
}

pub fn projection_matrix(width: u32, height: u32) -> Matrix4<f32> {
    let fov = 90.0 * 3.14 / 180.0;
    let aspect = width as f32 / height as f32;
    let near = 0.1;
    let far = 100.0;
    let f = 1.0 / ((fov / 2.0) as f32).tan();

    Matrix4::from([
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ])
}
