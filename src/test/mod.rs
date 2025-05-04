use std::path::PathBuf;
use yun_lib::interpreter::Interpreter;

#[test]
fn hello_world() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/hello_world.yun"))
            .is_ok()
    );
}

#[test]
fn vars() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/vars.yun"))
            .is_ok()
    );
}

#[test]
fn blocks() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/blocks.yun"))
            .is_ok()
    );
}

#[test]
fn loops() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/loops.yun"))
            .is_ok()
    );
}

#[test]
#[should_panic]
fn panic() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/panic.yun"))
            .is_ok()
    )
}

#[test]
fn functions() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/functions.yun"))
            .is_ok()
    )
}

#[test]
fn fib() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/fib.yun"))
            .is_ok()
    )
}

#[test]
fn benchmark() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/benchmark.yun"))
            .is_ok()
    )
}

#[test]
fn closures() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/closures.yun"))
            .is_ok()
    )
}

#[test]
fn closures_2() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/closures2.yun"))
            .is_ok()
    )
}

#[test]
fn class() {
    assert!(
        Interpreter::default()
            .run_test(&PathBuf::from("./examples/class.yun"))
            .is_ok()
    )
}
