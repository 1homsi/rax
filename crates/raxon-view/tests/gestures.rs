//! Gesture modifiers (tap/long-press/double-tap) on arbitrary views.

use std::cell::Cell;
use std::rc::Rc;

use raxon_dom::{Event, GestureKind, Host, Mutation, RecordingBackend, Tree, WidgetId};
use raxon_view::{text, View, ViewExt};

fn build<V: View>(view: V) -> (Tree, Rc<std::cell::RefCell<Vec<Mutation>>>, WidgetId) {
    let backend = RecordingBackend::new();
    let log = backend.log();
    let mut tree = Tree::new(Host::new(backend));
    let id = view.build(&mut tree);
    (tree, log, id)
}

#[test]
fn on_tap_enables_a_recognizer_and_fires() {
    let tapped = Rc::new(Cell::new(0));
    let t2 = tapped.clone();
    let (mut tree, log, id) = build(text("card").on_tap(move || t2.set(t2.get() + 1)));

    // The backend was told to attach a tap recognizer to this (non-button) view.
    assert!(log.borrow().contains(&Mutation::AddGesture {
        id,
        gesture: GestureKind::Tap
    }));

    tree.dispatch(&Event::Tap { target: id });
    assert_eq!(tapped.get(), 1, "tap handler fired on a plain text view");
}

#[test]
fn long_press_and_double_tap_route_to_their_handlers() {
    let long = Rc::new(Cell::new(false));
    let dbl = Rc::new(Cell::new(false));
    let (l2, d2) = (long.clone(), dbl.clone());
    let (mut tree, log, id) = build(
        text("x")
            .on_long_press(move || l2.set(true))
            .on_double_tap(move || d2.set(true)),
    );

    let g = log.borrow();
    assert!(g.contains(&Mutation::AddGesture {
        id,
        gesture: GestureKind::LongPress
    }));
    assert!(g.contains(&Mutation::AddGesture {
        id,
        gesture: GestureKind::DoubleTap
    }));
    drop(g);

    tree.dispatch(&Event::LongPress { target: id });
    tree.dispatch(&Event::DoubleTap { target: id });
    assert!(long.get());
    assert!(dbl.get());
}

#[test]
fn on_pan_enables_recognizer_and_reports_translation() {
    use raxon_core::Point;
    use raxon_dom::GesturePhase;

    let last = Rc::new(std::cell::RefCell::new(None));
    let l2 = last.clone();
    let (mut tree, log, id) = build(text("drag").on_pan(move |info| *l2.borrow_mut() = Some(info)));

    assert!(log.borrow().contains(&Mutation::AddGesture {
        id,
        gesture: GestureKind::Pan
    }));

    tree.dispatch(&Event::PanChanged {
        target: id,
        translation: Point::new(12.0, -4.0),
        velocity: Point::new(100.0, 0.0),
        phase: GesturePhase::Changed,
    });

    let info = last.borrow().expect("pan handler fired");
    assert_eq!(info.translation, Point::new(12.0, -4.0));
    assert_eq!(info.phase, GesturePhase::Changed);
}
