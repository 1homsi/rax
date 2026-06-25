//! Behavioural tests for signals, memos, effects, batching, and tracking.
//!
//! Each `#[test]` runs on its own thread, so the per-thread default runtime is
//! isolated per test.

mod common;
use common::recorder;

use std::cell::RefCell;
use std::rc::Rc;

use rax_reactive::*;

#[test]
fn signal_get_set_roundtrip() {
    let s = create_signal(10);
    assert_eq!(s.get(), 10);
    s.set(20);
    assert_eq!(s.get(), 20);
}

#[test]
fn effect_runs_once_on_creation() {
    let (log, sink) = recorder::<i32>();
    let s = create_signal(1);
    create_effect(move || sink(s.get()));
    assert_eq!(*log.borrow(), vec![1]);
}

#[test]
fn effect_reruns_when_dependency_changes() {
    let (log, sink) = recorder::<i32>();
    let s = create_signal(1);
    create_effect(move || sink(s.get()));
    s.set(2);
    s.set(3);
    assert_eq!(*log.borrow(), vec![1, 2, 3]);
}

#[test]
fn unchanged_set_does_not_rerun_effect() {
    let (log, sink) = recorder::<i32>();
    let s = create_signal(1);
    create_effect(move || sink(s.get()));
    s.set(1);
    s.set(1);
    assert_eq!(
        *log.borrow(),
        vec![1],
        "PartialEq change detection should suppress reruns"
    );
}

#[test]
fn memo_caches_and_recomputes_only_when_input_changes() {
    let compute_count = Rc::new(RefCell::new(0));
    let s = create_signal(2);
    let m = {
        let compute_count = compute_count.clone();
        create_memo(move || {
            *compute_count.borrow_mut() += 1;
            s.get() * 10
        })
    };

    assert_eq!(*compute_count.borrow(), 0, "memo is lazy");
    assert_eq!(m.get(), 20);
    assert_eq!(m.get(), 20);
    assert_eq!(*compute_count.borrow(), 1, "cached");

    s.set(3);
    assert_eq!(m.get(), 30);
    assert_eq!(*compute_count.borrow(), 2);
}

#[test]
fn memo_suppresses_downstream_when_value_unchanged() {
    let (log, sink) = recorder::<bool>();
    let n = create_signal(2);
    let is_even = create_memo(move || n.get() % 2 == 0);
    create_effect(move || sink(is_even.get()));

    assert_eq!(*log.borrow(), vec![true]);
    n.set(4);
    assert_eq!(
        *log.borrow(),
        vec![true],
        "still even -> no downstream rerun"
    );
    n.set(5);
    assert_eq!(*log.borrow(), vec![true, false]);
}

#[test]
fn diamond_dependency_computes_each_node_once_and_is_glitch_free() {
    let b_runs = Rc::new(RefCell::new(0));
    let c_runs = Rc::new(RefCell::new(0));
    let (d_log, d_sink) = recorder::<i32>();

    let a = create_signal(1);
    let b = {
        let b_runs = b_runs.clone();
        create_memo(move || {
            *b_runs.borrow_mut() += 1;
            a.get() + 1
        })
    };
    let c = {
        let c_runs = c_runs.clone();
        create_memo(move || {
            *c_runs.borrow_mut() += 1;
            a.get() * 10
        })
    };
    create_effect(move || d_sink(b.get() + c.get()));

    assert_eq!(*d_log.borrow(), vec![12]);
    let (b0, c0) = (*b_runs.borrow(), *c_runs.borrow());

    a.set(2);

    assert_eq!(
        *d_log.borrow(),
        vec![12, 23],
        "no glitch / stale-input value"
    );
    assert_eq!(*b_runs.borrow(), b0 + 1, "b recomputed exactly once");
    assert_eq!(*c_runs.borrow(), c0 + 1, "c recomputed exactly once");
}

#[test]
fn batch_coalesces_writes_into_a_single_effect_run() {
    let (log, sink) = recorder::<i32>();
    let x = create_signal(0);
    let y = create_signal(0);
    create_effect(move || sink(x.get() + y.get()));
    assert_eq!(*log.borrow(), vec![0]);

    batch(|| {
        x.set(1);
        y.set(2);
    });

    assert_eq!(*log.borrow(), vec![0, 3]);
}

#[test]
fn dynamic_dependencies_are_retracked_each_run() {
    let (log, sink) = recorder::<i32>();
    let use_a = create_signal(true);
    let a = create_signal(100);
    let b = create_signal(200);
    create_effect(move || sink(if use_a.get() { a.get() } else { b.get() }));
    assert_eq!(*log.borrow(), vec![100]);

    b.set(201);
    assert_eq!(*log.borrow(), vec![100], "b not tracked yet");

    use_a.set(false);
    assert_eq!(*log.borrow(), vec![100, 201]);

    a.set(101);
    assert_eq!(*log.borrow(), vec![100, 201], "a no longer tracked");
    b.set(202);
    assert_eq!(*log.borrow(), vec![100, 201, 202]);
}

#[test]
fn untrack_reads_without_subscribing() {
    let (log, sink) = recorder::<i32>();
    let tracked = create_signal(1);
    let hidden = create_signal(10);
    create_effect(move || sink(tracked.get() + untrack(|| hidden.get())));
    assert_eq!(*log.borrow(), vec![11]);

    hidden.set(20);
    assert_eq!(
        *log.borrow(),
        vec![11],
        "untracked read is not a dependency"
    );

    tracked.set(2);
    assert_eq!(
        *log.borrow(),
        vec![11, 22],
        "rerun re-reads hidden's current value"
    );
}

#[test]
fn disposed_effect_stops_running() {
    let (log, sink) = recorder::<i32>();
    let s = create_signal(1);
    let eff = create_effect(move || sink(s.get()));
    s.set(2);
    eff.dispose();
    s.set(3);
    assert_eq!(*log.borrow(), vec![1, 2], "no run after dispose");
}

#[test]
fn update_mutates_in_place_and_notifies() {
    let (log, sink) = recorder::<i32>();
    let s = create_signal(0);
    create_effect(move || sink(s.with(|v| *v)));
    s.update(|v| *v += 5);
    assert_eq!(*log.borrow(), vec![0, 5]);
}

#[test]
fn with_reads_by_reference_without_clone() {
    let s = create_signal(String::from("hi"));
    assert_eq!(s.with(|v| v.len()), 2);
    s.set(String::from("hello"));
    assert_eq!(s.with(|v| v.len()), 5);
}
