//! Context: dependency injection down the ownership tree.
//!
//! [`provide_context`] stores a value at the current scope (the owner active
//! during [`create_root`](super::create_root) or an effect). [`use_context`]
//! looks it up by type, walking outward to the nearest enclosing scope that
//! provided it. This is how cross-cutting values (themes, the navigator, app
//! services) reach deep widgets without threading them through every call.

use core::any::TypeId;

use super::runtime::{current_runtime, with_rt};

/// Provides `value` at the current scope. A later [`use_context::<T>`] in this
/// scope or any descendant scope will find it. Re-providing the same type at the
/// same scope replaces it.
pub fn provide_context<T: 'static>(value: T) {
    let rt = current_runtime();
    with_rt(rt, |r| {
        r.provide_context(TypeId::of::<T>(), Box::new(value))
    });
}

/// Looks up the nearest provided value of type `T`, or `None` if none is in
/// scope. Returns a clone.
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    let rt = current_runtime();
    with_rt(rt, |r| {
        r.lookup_context(TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    })
    .flatten()
}

/// Like [`use_context`] but panics with a clear message if the context is
/// missing — for values an app guarantees are always provided (e.g. a theme).
pub fn expect_context<T: Clone + 'static>() -> T {
    use_context::<T>().unwrap_or_else(|| {
        panic!(
            "expect_context::<{}>() called with no provider in scope",
            core::any::type_name::<T>()
        )
    })
}
