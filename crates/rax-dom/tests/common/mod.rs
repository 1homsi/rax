//! Shared harness: a `Tree` wired to a `RecordingBackend`, plus its log handle.

use std::cell::RefCell;
use std::rc::Rc;

use rax_dom::{Host, Mutation, RecordingBackend, Tree};

/// Builds a tree backed by a recording backend, returning the tree and a shared
/// handle to the mutation log for assertions.
pub fn harness() -> (Tree, Rc<RefCell<Vec<Mutation>>>) {
    let backend = RecordingBackend::new();
    let log = backend.log();
    (Tree::new(Host::new(backend)), log)
}
