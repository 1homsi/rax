//! Runtime-isolation tests (R1 fix): multiple independent reactive graphs on one
//! thread, and graceful behaviour when a runtime is disposed.

mod common;
use common::recorder;

use raxon_reactive::*;

#[test]
fn two_runtimes_are_independent() {
    let rt_a = Runtime::new();
    let rt_b = Runtime::new();
    let (log_a, sink_a) = recorder::<i32>();
    let (log_b, sink_b) = recorder::<i32>();

    let sa = rt_a.enter(|| {
        let s = create_signal(0);
        create_effect(move || sink_a(s.get()));
        s
    });
    let sb = rt_b.enter(|| {
        let s = create_signal(0);
        create_effect(move || sink_b(s.get()));
        s
    });

    assert_eq!(*log_a.borrow(), vec![0]);
    assert_eq!(*log_b.borrow(), vec![0]);

    // A write resolves to its signal's home runtime regardless of what's entered.
    sa.set(1);
    assert_eq!(*log_a.borrow(), vec![0, 1]);
    assert_eq!(*log_b.borrow(), vec![0], "runtime B untouched by A's write");

    sb.set(5);
    assert_eq!(*log_b.borrow(), vec![0, 5]);
    assert_eq!(
        *log_a.borrow(),
        vec![0, 1],
        "runtime A untouched by B's write"
    );
}

#[test]
fn writing_to_a_disposed_runtime_is_a_noop_not_a_panic() {
    let (log, sink) = recorder::<i32>();

    let s = {
        let rt = Runtime::new();
        // `rt` is dropped at the end of this block -> its graph is disposed.
        rt.enter(|| {
            let s = create_signal(0);
            create_effect(move || sink(s.get()));
            s
        })
    };

    assert_eq!(*log.borrow(), vec![0]);
    s.set(1); // runtime gone: must be a silent no-op, never a panic.
    assert_eq!(*log.borrow(), vec![0]);
}

#[test]
fn default_runtime_is_independent_of_explicit_runtimes() {
    let (log_default, sink_default) = recorder::<i32>();
    let default_signal = create_signal(0); // no enter -> thread default runtime
    create_effect(move || sink_default(default_signal.get()));

    let rt = Runtime::new();
    let explicit_signal = rt.enter(|| create_signal(100));

    explicit_signal.set(101); // touches only `rt`
    assert_eq!(
        *log_default.borrow(),
        vec![0],
        "default runtime effect not disturbed"
    );

    default_signal.set(1);
    assert_eq!(*log_default.borrow(), vec![0, 1]);
}
