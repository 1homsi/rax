//! Propagation control primitives: [`batch`] and [`untrack`].

use crate::runtime::{self, current_runtime};

/// Batches multiple writes so dependent effects run **once** at the end instead
/// of after each write. Returns whatever `f` returns. Operates on the current
/// runtime.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    let rt = current_runtime();
    runtime::enter_batch(rt);
    let out = f();
    if runtime::leave_batch(rt) {
        runtime::flush_effects(rt);
    }
    out
}

/// Runs `f` *without* tracking any reads as dependencies of the current
/// computation. Reads still return current values; they just don't subscribe.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    let rt = current_runtime();
    let prev = runtime::take_observer(rt);
    let out = f();
    runtime::restore_observer(rt, prev);
    out
}
