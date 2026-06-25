//! Ownership-tree tests (R1 fix): scopes dispose what they create, and a
//! re-running effect disposes the reactivity it created on its previous run.
//! These behaviours were impossible under the old global-singleton design.

mod common;
use common::recorder;

use rax_reactive::*;

#[test]
fn create_root_returns_value_and_scope() {
    let (value, scope) = create_root(|| 42);
    assert_eq!(value, 42);
    scope.dispose();
}

#[test]
fn disposing_a_scope_stops_effects_created_within_it() {
    let (log, sink) = recorder::<i32>();
    // Signal lives outside the scope, so it survives to trigger reruns.
    let s = create_signal(0);

    let (_, scope) = create_root(|| {
        create_effect(move || sink(s.get()));
    });
    assert_eq!(*log.borrow(), vec![0]);

    s.set(1);
    assert_eq!(*log.borrow(), vec![0, 1]);

    scope.dispose();
    s.set(2);
    assert_eq!(
        *log.borrow(),
        vec![0, 1],
        "scope disposal stops its effects"
    );
}

#[test]
fn rerunning_effect_disposes_its_previous_nested_effects() {
    // The classic leak: an effect that creates a child effect on every run.
    // Without an ownership tree, every run leaks another live child.
    let (log, sink) = recorder::<(i32, i32)>();
    let a = create_signal(0);
    let b = create_signal(0);

    create_effect(move || {
        let av = a.get();
        let sink = sink.clone();
        // Nested effect, recreated each outer run; must be disposed on rerun.
        create_effect(move || sink((av, b.get())));
    });

    // outer run -> inner1 -> (0,0)
    assert_eq!(*log.borrow(), vec![(0, 0)]);

    b.set(1); // inner1 reruns
    assert_eq!(*log.borrow(), vec![(0, 0), (0, 1)]);

    a.set(5); // outer reruns: dispose inner1, create inner2 -> (5,1)
    assert_eq!(*log.borrow(), vec![(0, 0), (0, 1), (5, 1)]);

    b.set(2); // ONLY inner2 should fire; inner1 must be gone
    assert_eq!(
        *log.borrow(),
        vec![(0, 0), (0, 1), (5, 1), (5, 2)],
        "a leaked inner1 would add an extra (0, 2) entry"
    );
}

#[test]
fn nested_scopes_dispose_recursively() {
    let (log, sink) = recorder::<&'static str>();
    let trigger = create_signal(0);

    let (inner_scope, outer_scope) = {
        let mut captured_inner = None;
        let (_, outer) = create_root(|| {
            // An inner scope nested inside the outer one.
            let (_, inner) = create_root(|| {
                let sink = sink.clone();
                create_effect(move || {
                    trigger.get();
                    sink("inner");
                });
            });
            captured_inner = Some(inner);
        });
        (captured_inner.unwrap(), outer)
    };

    assert_eq!(*log.borrow(), vec!["inner"]);
    trigger.set(1);
    assert_eq!(*log.borrow(), vec!["inner", "inner"]);

    // Disposing the OUTER scope must also dispose the nested inner scope.
    outer_scope.dispose();
    trigger.set(2);
    assert_eq!(
        *log.borrow(),
        vec!["inner", "inner"],
        "outer disposal cascaded to inner"
    );

    // Disposing the already-freed inner scope is a harmless no-op.
    inner_scope.dispose();
}
