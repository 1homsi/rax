//! Inbound events (R3): routing, bubbling, the queue/drain path, and the full
//! round trip — a platform event becomes a signal write becomes one mutation.

mod common;
use common::harness;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rax_dom::*;
use rax_reactive::create_signal;

#[test]
fn tap_handler_writes_signal_and_produces_one_mutation() {
    // The end-to-end thesis for the seam: native event in -> handler -> signal
    // -> reactive effect -> exactly one mutation out.
    let (mut tree, log) = harness();
    let count = create_signal(0);
    let label = tree.create_text();
    tree.bind(label, move || Attribute::Text(count.get().to_string()));
    tree.on(label, EventKind::Tap, move |_| count.update(|c| *c += 1));

    log.borrow_mut().clear();
    tree.dispatch(&Event::Tap { target: label });

    assert_eq!(
        *log.borrow(),
        vec![Mutation::SetAttribute {
            id: label,
            attr: Attribute::Text("1".into())
        }]
    );
}

#[test]
fn events_bubble_from_target_up_to_ancestors() {
    let (mut tree, _log) = harness();
    let parent = tree.create_view();
    let child = tree.create_text();
    tree.append(parent, child);

    let handled_on_parent = Rc::new(Cell::new(false));
    {
        let flag = handled_on_parent.clone();
        tree.on(parent, EventKind::Tap, move |_| flag.set(true));
    }

    // The child has no Tap handler; the event must bubble to the parent.
    tree.dispatch(&Event::Tap { target: child });
    assert!(
        handled_on_parent.get(),
        "tap on child should bubble to parent"
    );
}

#[test]
fn multiple_handlers_run_in_registration_order() {
    let (mut tree, _log) = harness();
    let w = tree.create_view();
    let order = Rc::new(RefCell::new(Vec::<&str>::new()));
    {
        let order = order.clone();
        tree.on(w, EventKind::Tap, move |_| order.borrow_mut().push("first"));
    }
    {
        let order = order.clone();
        tree.on(w, EventKind::Tap, move |_| {
            order.borrow_mut().push("second")
        });
    }

    tree.dispatch(&Event::Tap { target: w });
    assert_eq!(*order.borrow(), vec!["first", "second"]);
}

#[test]
fn queued_events_are_only_delivered_on_drain() {
    let (mut tree, _log) = harness();
    let pressed = Rc::new(Cell::new(0));
    {
        let pressed = pressed.clone();
        tree.on_global(EventKind::BackPressed, move |_| {
            pressed.set(pressed.get() + 1)
        });
    }

    // A backend (possibly another thread) enqueues via the Send sink.
    let sink = tree.event_sink();
    sink.dispatch(Event::BackPressed);
    sink.dispatch(Event::BackPressed);
    assert_eq!(
        pressed.get(),
        0,
        "queued events are not delivered until drained"
    );

    tree.drain_events();
    assert_eq!(
        pressed.get(),
        2,
        "drain delivers all queued events in order"
    );
}

#[test]
fn handlers_are_gone_after_the_widget_is_removed() {
    let (mut tree, _log) = harness();
    let count = create_signal(0);
    let label = tree.create_text();
    tree.on(label, EventKind::Tap, move |_| count.update(|c| *c += 1));

    tree.remove(label);
    // Target no longer exists: dispatch finds no handler chain and does nothing.
    tree.dispatch(&Event::Tap { target: label });
    assert_eq!(count.get(), 0, "removed widget's handler must not fire");
}

#[test]
fn cross_thread_event_sink_delivers_on_drain() {
    let (mut tree, _log) = harness();
    let got = Rc::new(Cell::new(false));
    {
        let got = got.clone();
        tree.on_global(EventKind::BackPressed, move |_| got.set(true));
    }

    let sink = tree.event_sink();
    std::thread::spawn(move || sink.dispatch(Event::BackPressed))
        .join()
        .unwrap();

    tree.drain_events();
    assert!(
        got.get(),
        "event enqueued from another thread is delivered on the UI thread"
    );
}
