use std::cell::RefCell;
use std::rc::Rc;

use crate::reactive::{create_effect, create_root, create_signal};
use futures::channel::oneshot;

use super::{create_resource, run_until_stalled, spawn_local, ResourceState};

#[test]
fn resource_resolves_after_pumping() {
    let (res, scope) = create_root(|| create_resource(async { Ok::<i32, String>(42) }));
    assert!(res.loading());
    run_until_stalled();
    assert_eq!(res.data(), Some(42));
    scope.dispose();
}

#[test]
fn resource_failure_is_captured() {
    let (res, scope) = create_root(|| create_resource(async { Err::<i32, String>("boom".into()) }));
    run_until_stalled();
    assert_eq!(res.error().as_deref(), Some("boom"));
    scope.dispose();
}

#[test]
fn resource_pending_until_future_completes() {
    let (tx, rx) = oneshot::channel::<i32>();
    let (res, scope) = create_root(|| {
        create_resource(async move { rx.await.map_err(|_| "canceled".to_string()) })
    });

    run_until_stalled();
    assert!(res.loading(), "still loading before the value arrives");

    tx.send(7).unwrap();
    run_until_stalled();
    assert_eq!(res.data(), Some(7));
    scope.dispose();
}

#[test]
fn resource_drives_an_effect_reactively() {
    let log: Rc<RefCell<Vec<ResourceState<i32>>>> = Rc::new(RefCell::new(Vec::new()));
    let log2 = log.clone();
    let (_res, scope) = create_root(move || {
        let res = create_resource(async { Ok::<i32, String>(1) });
        create_effect(move || log2.borrow_mut().push(res.get()));
    });

    assert_eq!(log.borrow()[0], ResourceState::Loading);
    run_until_stalled();
    assert_eq!(*log.borrow().last().unwrap(), ResourceState::Ready(1));
    scope.dispose();
}

#[test]
fn resource_handles_can_be_stored_in_signals() {
    let ((), scope) = create_root(|| {
        let first = create_resource(async { Ok::<i32, String>(1) });
        let second = create_resource(async { Ok::<i32, String>(2) });
        let slot = create_signal(first);

        assert!(slot.get() == first);
        assert!(first != second);

        slot.set(second);
        assert!(slot.get() == second);
    });
    scope.dispose();
}

#[test]
fn spawn_local_runs_tasks() {
    let ran = Rc::new(RefCell::new(false));
    let r2 = ran.clone();
    spawn_local(async move {
        *r2.borrow_mut() = true;
    });
    assert!(!*ran.borrow());
    run_until_stalled();
    assert!(*ran.borrow());
}
