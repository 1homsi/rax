//! The render seam: the `Backend` trait every platform implements, and a shared
//! `Host` handle that reactive effects use to emit mutations.

use std::cell::RefCell;
use std::rc::Rc;

use crate::mutation::Mutation;

/// The single trait a platform backend implements.
///
/// `apply` receives each [`Mutation`] as it is produced. Android implements it
/// over JNI, iOS over the Objective-C runtime, and tests over a `Vec`. This one
/// method *is* the entire platform contract for rendering.
pub trait Backend {
    /// Apply one mutation to the native view tree.
    fn apply(&mut self, mutation: Mutation);
}

/// A cheap, cloneable handle to the active backend.
///
/// Reactive effects are `'static` closures living in the reactive runtime, so
/// they cannot borrow the tree. They instead capture a `Host` (an `Rc`) and emit
/// through it. This is the one piece of shared mutable state in the render path,
/// and it is deliberately tiny.
#[derive(Clone)]
pub struct Host {
    backend: Rc<RefCell<dyn Backend>>,
}

impl Host {
    /// Wraps a backend in a shareable host handle.
    pub fn new<B: Backend + 'static>(backend: B) -> Host {
        Host {
            backend: Rc::new(RefCell::new(backend)),
        }
    }

    /// Emits a single mutation to the backend.
    pub fn emit(&self, mutation: Mutation) {
        self.backend.borrow_mut().apply(mutation);
    }
}

/// A test/inspector backend that records every mutation into a shared log.
///
/// Clone the [`log`](RecordingBackend::log) handle before moving the backend
/// into a [`Host`] to assert on the recorded stream afterwards.
pub struct RecordingBackend {
    log: Rc<RefCell<Vec<Mutation>>>,
}

impl Default for RecordingBackend {
    fn default() -> Self {
        RecordingBackend::new()
    }
}

impl RecordingBackend {
    /// Creates an empty recording backend.
    pub fn new() -> Self {
        RecordingBackend {
            log: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// A shared handle to the recorded mutation log.
    pub fn log(&self) -> Rc<RefCell<Vec<Mutation>>> {
        self.log.clone()
    }
}

impl Backend for RecordingBackend {
    fn apply(&mut self, mutation: Mutation) {
        self.log.borrow_mut().push(mutation);
    }
}
