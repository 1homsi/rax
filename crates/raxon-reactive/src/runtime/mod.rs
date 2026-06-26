//! Runtime ownership and isolation.
//!
//! Each [`Runtime`] is an isolated reactive graph. Multiple can coexist on one
//! thread (multi-window, an embedded inspector, prerender-for-test), which the
//! original global-singleton design made impossible. Handles carry the id of
//! their home runtime, so an operation always resolves to the right graph.
//!
//! Ergonomics are preserved: code that never mentions a `Runtime` operates on a
//! lazily-created **per-thread default**, so the simple API still "just works".
//!
//! [`create_root`] / [`Scope`] expose the **ownership tree**: nodes created
//! while a scope is active are disposed together, and a re-running effect first
//! disposes everything it created last time — closing the leak the singleton had.

mod engine;
mod node;
mod reactor;

pub(crate) use engine::{
    flush_effects, read_cloned, read_with, set_value, update_if_necessary, update_value,
};
pub(crate) use node::Node;
pub(crate) use reactor::Reactor;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use raxon_core::Index;

/// Identifies a [`Runtime`] within the current thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeId(pub(crate) u32);

thread_local! {
    /// All runtimes on this thread, indexed by `RuntimeId.0`. `None` = disposed.
    static REGISTRY: RefCell<Vec<Option<Rc<RefCell<Reactor>>>>> = const { RefCell::new(Vec::new()) };
    /// Stack of entered runtimes; the top is "current".
    static CURRENT: RefCell<Vec<RuntimeId>> = const { RefCell::new(Vec::new()) };
    /// The lazily-created per-thread default runtime.
    static DEFAULT: Cell<Option<RuntimeId>> = const { Cell::new(None) };
}

struct RuntimeEnterGuard;

impl Drop for RuntimeEnterGuard {
    fn drop(&mut self) {
        CURRENT.with(|c| {
            c.borrow_mut().pop();
        });
    }
}

struct OwnerGuard {
    rt: RuntimeId,
    prev_owner: Option<Index>,
}

impl Drop for OwnerGuard {
    fn drop(&mut self) {
        with_rt(self.rt, |r| r.owner = self.prev_owner);
    }
}

/// Resolves `id` to its reactor and runs `f` against it. Returns `None` if the
/// runtime has been disposed. The `REGISTRY` borrow is released before the
/// reactor borrow so reactor ops may safely re-enter the registry.
pub(crate) fn with_rt<R>(id: RuntimeId, f: impl FnOnce(&mut Reactor) -> R) -> Option<R> {
    let rc = REGISTRY.with(|reg| {
        reg.borrow()
            .get(id.0 as usize)
            .and_then(|slot| slot.clone())
    });
    rc.map(|rc| f(&mut rc.borrow_mut()))
}

fn alloc_runtime() -> RuntimeId {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = RuntimeId(reg.len() as u32);
        reg.push(Some(Rc::new(RefCell::new(Reactor::new()))));
        id
    })
}

fn free_runtime(id: RuntimeId) {
    REGISTRY.with(|reg| {
        if let Some(slot) = reg.borrow_mut().get_mut(id.0 as usize) {
            *slot = None;
        }
    });
    DEFAULT.with(|d| {
        if d.get() == Some(id) {
            d.set(None);
        }
    });
}

/// The runtime new nodes are created in: the entered one, or the per-thread
/// default (created on first use).
pub(crate) fn current_runtime() -> RuntimeId {
    if let Some(id) = CURRENT.with(|c| c.borrow().last().copied()) {
        return id;
    }
    DEFAULT.with(|d| match d.get() {
        Some(id) => id,
        None => {
            let id = alloc_runtime();
            d.set(Some(id));
            id
        }
    })
}

/// An isolated reactive graph.
///
/// Drop disposes the entire graph. Use [`enter`](Runtime::enter) to make this
/// runtime current for a closure, so `create_signal`/`create_effect`/etc. inside
/// it belong to this runtime rather than the thread default.
pub struct Runtime {
    id: RuntimeId,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new()
    }
}

impl Runtime {
    /// Creates a fresh, empty runtime.
    pub fn new() -> Self {
        Runtime {
            id: alloc_runtime(),
        }
    }

    /// Makes this runtime current for the duration of `f`.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        CURRENT.with(|c| c.borrow_mut().push(self.id));
        let _guard = RuntimeEnterGuard;
        f()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        free_runtime(self.id);
    }
}

/// A disposable ownership scope. Dropping the handle does **not** dispose the
/// scope (so it can outlive a stack frame); call [`dispose`](Scope::dispose)
/// explicitly, typically tied to a UI element's lifetime.
pub struct Scope {
    rt: RuntimeId,
    owner: Index,
}

impl Scope {
    /// Disposes the scope and everything created within it (effects, memos,
    /// signals, and nested scopes), recursively.
    pub fn dispose(self) {
        with_rt(self.rt, |r| r.dispose(self.owner));
    }
}

/// Runs `f` inside a fresh ownership scope, returning its result and a [`Scope`]
/// handle to dispose everything `f` created. The reactive graph created here
/// will leak until the scope is disposed — this is the explicit root that app
/// and test code should establish.
pub fn create_root<T>(f: impl FnOnce() -> T) -> (T, Scope) {
    let rt = current_runtime();
    let owner = with_rt(rt, |r| r.insert_node(Node::scope())).expect("runtime exists");

    let prev_owner = with_rt(rt, |r| {
        let prev = r.owner;
        r.owner = Some(owner);
        prev
    })
    .flatten();

    let guard = OwnerGuard { rt, prev_owner };

    let out = f();

    drop(guard);
    (out, Scope { rt, owner })
}

// Batch depth control used by `crate::control::batch`.
pub(crate) fn enter_batch(rt: RuntimeId) {
    with_rt(rt, |r| r.batch_depth += 1);
}

/// Decrements batch depth; returns `true` if effects should now flush.
pub(crate) fn leave_batch(rt: RuntimeId) -> bool {
    with_rt(rt, |r| {
        r.batch_depth = r.batch_depth.saturating_sub(1);
        r.batch_depth == 0
    })
    .unwrap_or(false)
}

pub(crate) fn take_observer(rt: RuntimeId) -> Option<Index> {
    with_rt(rt, |r| r.observer.take()).flatten()
}

pub(crate) fn restore_observer(rt: RuntimeId, prev: Option<Index>) {
    with_rt(rt, |r| r.observer = prev);
}
