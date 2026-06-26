//! UI-thread async for `rax`: a single-threaded executor and an async-aware
//! [`Resource`].
//!
//! Futures here run on the **UI thread** (where signals live), so awaiting and
//! then writing a signal is safe and needs no marshaling. The app pumps the
//! executor once per frame via [`run_until_stalled`]; tests pump it directly.
//!
//! The executor wraps `futures`' `LocalPool` (well-tested wakers — no unsafe in
//! this crate). A cloned `LocalSpawner` lives in its own thread-local so
//! [`spawn_local`] never re-enters the pool's borrow, even when called from
//! inside a running task.
//!
//! ```
//! use raxon::async_rt::{create_resource, run_until_stalled, ResourceState};
//! use raxon::reactive::create_root;
//!
//! let (res, scope) = create_root(|| create_resource(async { Ok::<i32, String>(42) }));
//! assert!(matches!(res.get(), ResourceState::Loading));
//! run_until_stalled();
//! assert_eq!(res.data(), Some(42));
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::future::Future;

use futures::executor::{LocalPool, LocalSpawner};
use futures::task::LocalSpawnExt;
use crate::reactive::{create_signal, Signal};

thread_local! {
    static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static SPAWNER: LocalSpawner = POOL.with(|p| p.borrow().spawner());
}

/// Spawns a future onto the current thread's UI executor. It makes progress when
/// the executor is pumped ([`run_until_stalled`]).
pub fn spawn_local(future: impl Future<Output = ()> + 'static) {
    SPAWNER.with(|spawner| {
        let _ = spawner.spawn_local(future);
    });
}

/// Polls all ready tasks until none can make further progress. Called once per
/// frame by the runtime (and directly by tests).
pub fn run_until_stalled() {
    POOL.with(|pool| pool.borrow_mut().run_until_stalled());
}

/// The state of an async [`Resource`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T> {
    /// The future has not completed yet.
    Loading,
    /// The future resolved successfully.
    Ready(T),
    /// The future failed.
    Failed(String),
}

/// An async-aware reactive value: starts `Loading`, then becomes `Ready`/`Failed`
/// when its future completes. Reading [`get`](Resource::get) is reactive, so UI
/// that depends on it updates automatically.
pub struct Resource<T: 'static> {
    state: Signal<ResourceState<T>>,
}

// `Copy` handle (Signal is Copy), so `Resource` can be moved into closures.
impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Resource<T> {}

/// Creates a resource driven by `future`, which resolves to `Ok(value)` or
/// `Err(message)`. The future is spawned immediately on the UI executor.
pub fn create_resource<T, Fut>(future: Fut) -> Resource<T>
where
    T: Clone + 'static,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    let state = create_signal(ResourceState::Loading);
    spawn_local(async move {
        let next = match future.await {
            Ok(value) => ResourceState::Ready(value),
            Err(message) => ResourceState::Failed(message),
        };
        state.update(|s| *s = next);
    });
    Resource { state }
}

impl<T: Clone + 'static> Resource<T> {
    /// Reads the current state (a tracked read — readers update on change).
    pub fn get(&self) -> ResourceState<T> {
        self.state.get()
    }

    /// Whether the resource is still loading.
    pub fn loading(&self) -> bool {
        matches!(self.state.get(), ResourceState::Loading)
    }

    /// The value if ready, else `None`.
    pub fn data(&self) -> Option<T> {
        match self.state.get() {
            ResourceState::Ready(v) => Some(v),
            _ => None,
        }
    }

    /// The error message if failed, else `None`.
    pub fn error(&self) -> Option<String> {
        match self.state.get() {
            ResourceState::Failed(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
