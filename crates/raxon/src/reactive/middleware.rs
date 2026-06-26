//! Middleware / interceptors for signal writes.
//!
//! Register global middleware that runs on every signal set() call.
//! Useful for logging, persistence, devtools time-travel, etc.
//!
//! # Example
//! ```
//! use raxon::reactive::add_signal_middleware;
//! add_signal_middleware(|key, value| {
//!     println!("[signal] {key} = {value}");
//! });
//! ```

use std::cell::RefCell;

thread_local! {
    static MIDDLEWARES: RefCell<Vec<Box<dyn Fn(&str, &str)>>> = RefCell::new(vec![]);
}

/// Register a global signal middleware. Called on every `Signal::set` with
/// (signal_type_name, debug_value). Values are formatted with `{:?}` if Debug.
pub fn add_signal_middleware(f: impl Fn(&str, &str) + 'static) {
    MIDDLEWARES.with(|m| m.borrow_mut().push(Box::new(f)));
}

/// Remove all registered middlewares.
pub fn clear_signal_middlewares() {
    MIDDLEWARES.with(|m| m.borrow_mut().clear());
}

/// Called internally by signal infrastructure to notify middlewares.
/// Key is the type name, value is the debug string.
pub(crate) fn notify_middlewares(key: &str, value: &str) {
    MIDDLEWARES.with(|m| {
        for f in m.borrow().iter() {
            f(key, value);
        }
    });
}
