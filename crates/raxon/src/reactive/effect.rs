//! Effects: side-effecting sinks that re-run when their inputs change.

use super::runtime::{self, current_runtime, update_if_necessary, Node};

/// A handle to a running effect, used to dispose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect {
    rt: runtime::RuntimeId,
    key: crate::core::Index,
}

/// Creates an effect running `f`. It runs **once immediately** to establish its
/// dependencies, then re-runs whenever any input changes. The effect is owned by
/// the current scope, so it is disposed when that scope is.
pub fn create_effect<F: FnMut() + 'static>(f: F) -> Effect {
    let rt = current_runtime();
    let key =
        runtime::with_rt(rt, |r| r.insert_node(Node::effect(Box::new(f)))).expect("runtime exists");
    update_if_necessary(rt, key); // initial run
    Effect { rt, key }
}

impl Effect {
    /// Disposes the effect (and anything it owns): it stops re-running and is
    /// detached from its inputs.
    pub fn dispose(self) {
        runtime::with_rt(self.rt, |r| r.dispose(self.key));
    }
}
