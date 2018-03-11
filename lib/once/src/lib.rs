#![no_std]

#[cfg(test)]
extern crate std;

#[macro_export]
macro_rules! assert_first_call {
    () => {
        assert_first_call!("assertion failed: function called more than once");
    };

    ($($arg:tt)+) => {{
        use ::core::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
        static CALLED: AtomicBool = ATOMIC_BOOL_INIT;
        let called = CALLED.swap(true, Ordering::Relaxed);
        assert!(!called, $($arg)+);
    }};
}

#[test]
fn test_run_once() {
    fn once() {
        assert_first_call!();
    }

    once();
}

#[test]
fn test_run_once_two_funcs() {
    fn once1() {assert_first_call!();}
    fn once2() {assert_first_call!();}

    once1(); once2();
}

#[test]
#[should_panic]
fn test_run_twice() {
    fn once() {
        assert_first_call!();
    }

    once(); once();
}
