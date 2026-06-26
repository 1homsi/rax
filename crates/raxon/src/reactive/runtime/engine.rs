//! Propagation orchestration: the multi-step operations that must release the
//! reactor borrow because they run user closures which re-enter the reactor.
//!
//! Each function takes a [`RuntimeId`] and resolves it per chunk via
//! [`with_rt`](super::with_rt). The borrow is never held across user code.

use core::any::Any;

use crate::core::Index;

use super::node::{Compute, NodeState};
use super::{with_rt, RuntimeId};
use super::super::middleware::notify_middlewares;

fn state_of(rt: RuntimeId, key: Index) -> Option<NodeState> {
    with_rt(rt, |r| r.state_of(key)).flatten()
}

/// Ensures `key`'s value is current, recomputing only if a source actually
/// changed (the pull half of the algorithm).
pub(crate) fn update_if_necessary(rt: RuntimeId, key: Index) {
    match state_of(rt, key) {
        Some(NodeState::Clean) | None => return,
        Some(NodeState::Check) => {
            let sources = with_rt(rt, |r| r.nodes.get(key).map(|n| n.sources.clone()))
                .flatten()
                .unwrap_or_default();
            for s in sources {
                update_if_necessary(rt, s);
                if state_of(rt, key) == Some(NodeState::Dirty) {
                    break; // a source's recompute marked us Dirty; stop early
                }
            }
        }
        Some(NodeState::Dirty) => {}
    }

    if state_of(rt, key) == Some(NodeState::Dirty) {
        recompute(rt, key);
    }
    with_rt(rt, |r| {
        if let Some(n) = r.nodes.get_mut(key) {
            n.state = NodeState::Clean;
        }
    });
}

/// Re-runs a node's computation with a fresh dependency set. Disposes the
/// node's previously-owned children first (ownership cleanup), then runs user
/// code unborrowed, then commits.
fn recompute(rt: RuntimeId, key: Index) {
    enum Taken {
        Memo(super::node::MemoFn, Option<Box<dyn Any>>),
        Effect(super::node::EffectFn),
        Nothing,
    }

    // Phase 1 (borrowed): dispose old children, clear deps, install observer +
    // owner, take the closure (and previous value, for memos) out.
    let prep = with_rt(rt, |r| {
        r.dispose_children(key);
        r.clear_sources(key);
        let prev_observer = r.observer;
        let prev_owner = r.owner;
        r.observer = Some(key);
        r.owner = Some(key);
        let taken = match r.take_compute(key) {
            Compute::Memo(f) => Taken::Memo(f, r.nodes.get_mut(key).and_then(|n| n.value.take())),
            Compute::Effect(f) => Taken::Effect(f),
            Compute::None => Taken::Nothing,
        };
        (taken, prev_observer, prev_owner)
    });
    let Some((taken, prev_observer, prev_owner)) = prep else {
        return;
    };

    // Phase 2 (unborrowed): run user code, which re-enters the reactor to read.
    let (new_value, compute_back) = match taken {
        Taken::Memo(mut f, prev) => {
            let (value, changed) = f(prev);
            (Some((value, changed)), Compute::Memo(f))
        }
        Taken::Effect(mut f) => {
            f();
            (None, Compute::Effect(f))
        }
        Taken::Nothing => (None, Compute::None),
    };

    // Phase 3 (borrowed): restore observer/owner + closure, commit, mark deps.
    with_rt(rt, |r| {
        r.observer = prev_observer;
        r.owner = prev_owner;
        let observers = match r.nodes.get_mut(key) {
            Some(n) => {
                n.compute = compute_back;
                match new_value {
                    Some((value, changed)) => {
                        n.value = Some(value);
                        if changed {
                            n.observers.clone()
                        } else {
                            Vec::new()
                        }
                    }
                    None => Vec::new(),
                }
            }
            None => Vec::new(),
        };
        for o in observers {
            if let Some(on) = r.nodes.get_mut(o) {
                if on.state < NodeState::Dirty {
                    on.state = NodeState::Dirty;
                }
            }
        }
    });
}

/// Reads a node's value as `T`, ensuring freshness and tracking the dependency.
pub(crate) fn read_cloned<T: Clone + 'static>(rt: RuntimeId, key: Index) -> T {
    update_if_necessary(rt, key);
    with_rt(rt, |r| {
        if let Some(obs) = r.observer {
            r.subscribe(key, obs);
        }
        r.nodes
            .get(key)
            .and_then(|n| n.value.as_ref())
            .and_then(|v| v.downcast_ref::<T>())
            .expect("reactive node missing value or wrong type")
            .clone()
    })
    .expect("signal/memo read after its runtime was disposed")
}

/// Like [`read_cloned`] but hands a reference to `f` instead of cloning.
pub(crate) fn read_with<T: 'static, R>(rt: RuntimeId, key: Index, f: impl FnOnce(&T) -> R) -> R {
    update_if_necessary(rt, key);
    with_rt(rt, |r| {
        if let Some(obs) = r.observer {
            r.subscribe(key, obs);
        }
        let value = r
            .nodes
            .get(key)
            .and_then(|n| n.value.as_ref())
            .and_then(|v| v.downcast_ref::<T>())
            .expect("reactive node missing value or wrong type");
        f(value)
    })
    .expect("signal/memo read after its runtime was disposed")
}

/// Sets a signal's value with `PartialEq` change detection.
pub(crate) fn set_value<T: PartialEq + 'static>(rt: RuntimeId, key: Index, value: T) {
    let changed = with_rt(rt, |r| {
        r.nodes
            .get(key)
            .and_then(|n| n.value.as_ref())
            .and_then(|v| v.downcast_ref::<T>())
            .map(|old| *old != value)
            .unwrap_or(false)
    })
    .unwrap_or(false);
    if !changed {
        return;
    }
    notify_middlewares(std::any::type_name::<T>(), "<updated>");
    propagate_write(rt, key, Box::new(value));
}

/// Mutates a signal's value via `f` and unconditionally notifies dependents.
pub(crate) fn update_value<T: 'static>(rt: RuntimeId, key: Index, f: impl FnOnce(&mut T)) {
    let taken = with_rt(rt, |r| r.nodes.get_mut(key).and_then(|n| n.value.take())).flatten();
    let Some(mut boxed) = taken else { return };
    if let Some(v) = boxed.downcast_mut::<T>() {
        f(v);
    }
    notify_middlewares(std::any::type_name::<T>(), "<updated>");
    propagate_write(rt, key, boxed);
}

/// Stores `value` into `key`, notifies dependents, and flushes if not batching.
fn propagate_write(rt: RuntimeId, key: Index, value: Box<dyn Any>) {
    let should_flush = with_rt(rt, |r| {
        if let Some(n) = r.nodes.get_mut(key) {
            n.value = Some(value);
        }
        let observers = r
            .nodes
            .get(key)
            .map(|n| n.observers.clone())
            .unwrap_or_default();
        for o in observers {
            r.notify(o, NodeState::Dirty);
        }
        r.batch_depth == 0
    })
    .unwrap_or(false);
    if should_flush {
        flush_effects(rt);
    }
}

/// Drains the runtime's effect queue, running each scheduled effect. Writes made
/// inside an effect re-enqueue and are picked up by the same loop.
pub(crate) fn flush_effects(rt: RuntimeId) {
    loop {
        let next = with_rt(rt, |r| {
            if r.batch_depth == 0 {
                r.effect_queue.pop()
            } else {
                None
            }
        })
        .flatten();
        match next {
            Some(k) => update_if_necessary(rt, k),
            None => break,
        }
    }
}
