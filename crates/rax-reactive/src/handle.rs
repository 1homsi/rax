//! The typed, `Copy` public handles: [`Signal`] and [`Memo`], plus their
//! constructors. Each handle records the id of its home runtime so reads/writes
//! always resolve to the correct graph.

use core::any::Any;
use core::marker::PhantomData;

use rax_core::Index;

use crate::runtime::{
    self, current_runtime, read_cloned, read_with, set_value, update_value, Node,
};

/// A reactive source cell holding a value of type `T`.
///
/// Cheap `Copy` handle; clone it freely into `move` closures.
pub struct Signal<T> {
    rt: runtime::RuntimeId,
    key: Index,
    _ty: PhantomData<fn() -> T>,
}

// Manual impls so the *handle* is `Copy` regardless of whether `T` is.
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Signal<T> {}

/// A derived, memoized value of type `T`.
pub struct Memo<T> {
    rt: runtime::RuntimeId,
    key: Index,
    _ty: PhantomData<fn() -> T>,
}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Memo<T> {}

/// Creates a signal initialized to `value`, owned by the current scope/runtime.
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    let rt = current_runtime();
    let key = runtime::with_rt(rt, |r| r.insert_node(Node::signal(Box::new(value))))
        .expect("runtime exists");
    Signal {
        rt,
        key,
        _ty: PhantomData,
    }
}

/// Creates a **lazy** memo computing `f`: it does not run until first read, and
/// re-runs only when an input changes and it is read again. `T: PartialEq` lets
/// it suppress downstream updates when the recomputed value is unchanged.
pub fn create_memo<T, F>(mut f: F) -> Memo<T>
where
    T: PartialEq + 'static,
    F: FnMut() -> T + 'static,
{
    let compute = move |prev: Option<Box<dyn Any>>| -> (Box<dyn Any>, bool) {
        let next = f();
        let changed = match prev {
            Some(p) => *p.downcast::<T>().expect("memo value type is stable") != next,
            None => true,
        };
        (Box::new(next) as Box<dyn Any>, changed)
    };
    let rt = current_runtime();
    let key = runtime::with_rt(rt, |r| r.insert_node(Node::memo(Box::new(compute))))
        .expect("runtime exists");
    Memo {
        rt,
        key,
        _ty: PhantomData,
    }
}

impl<T: 'static> Signal<T> {
    /// Reads the value, tracking this signal as a dependency of the current
    /// computation. Returns an owned copy (`T: Clone`); use [`with`](Signal::with)
    /// to avoid cloning.
    pub fn get(self) -> T
    where
        T: Clone,
    {
        read_cloned::<T>(self.rt, self.key)
    }

    /// Reads the value by reference via `f`, tracking the dependency without
    /// cloning.
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        read_with::<T, R>(self.rt, self.key, f)
    }

    /// Sets the value, notifying dependents only if it changed (`T: PartialEq`).
    pub fn set(self, value: T)
    where
        T: PartialEq,
    {
        set_value::<T>(self.rt, self.key, value);
    }

    /// Mutates the value via `f` and always notifies dependents (no equality
    /// check). Use when `T` is not `PartialEq` or a change is certain.
    pub fn update(self, f: impl FnOnce(&mut T)) {
        update_value::<T>(self.rt, self.key, f);
    }
}

impl<T: 'static> Memo<T> {
    /// Reads the memoized value, recomputing if stale, and tracks the dependency.
    pub fn get(self) -> T
    where
        T: Clone,
    {
        read_cloned::<T>(self.rt, self.key)
    }

    /// Reads the memoized value by reference via `f`, recomputing if stale.
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        read_with::<T, R>(self.rt, self.key, f)
    }
}
