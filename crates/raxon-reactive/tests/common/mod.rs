//! Shared test helpers.

use std::cell::RefCell;
use std::rc::Rc;

/// A cheap recorder for capturing effect outputs across runs without `'static`
/// borrow gymnastics. Returns the shared log and a sink closure to push into it.
pub fn recorder<T: Clone + 'static>() -> (Rc<RefCell<Vec<T>>>, impl Fn(T) + Clone) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = {
        let log = log.clone();
        move |v: T| log.borrow_mut().push(v)
    };
    (log, sink)
}
