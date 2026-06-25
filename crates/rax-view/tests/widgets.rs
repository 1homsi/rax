//! Switch / slider / image widgets via the recording backend.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rax_dom::{Attribute, Event, Host, Mutation, RecordingBackend, TextSelection, Tree};
use rax_view::{image, segmented, slider, stepper, switch, text_input, View};

fn harness() -> (Tree, Rc<std::cell::RefCell<Vec<Mutation>>>) {
    let backend = RecordingBackend::new();
    let log = backend.log();
    (Tree::new(Host::new(backend)), log)
}

#[test]
fn switch_emits_initial_value_and_reports_toggles() {
    let (mut tree, log) = harness();
    let toggled = Rc::new(Cell::new(false));
    let t2 = toggled.clone();
    let id = switch(false, move |on| t2.set(on)).build(&mut tree);

    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::BoolValue(false)
    }));

    tree.dispatch(&Event::ValueChanged {
        target: id,
        value: 1.0,
    });
    assert!(toggled.get(), "switch reported on");
}

#[test]
fn slider_reports_value() {
    let (mut tree, log) = harness();
    let last = Rc::new(Cell::new(0.0_f32));
    let l2 = last.clone();
    let id = slider(0.25, move |v| l2.set(v)).build(&mut tree);

    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::FloatValue(0.25)
    }));

    tree.dispatch(&Event::ValueChanged {
        target: id,
        value: 0.8,
    });
    assert!((last.get() - 0.8).abs() < 1e-6, "slider reported new value");
}

#[test]
fn segmented_emits_titles_and_selection_and_reports_picks() {
    let (mut tree, log) = harness();
    let picked = Rc::new(Cell::new(usize::MAX));
    let p2 = picked.clone();
    let id = segmented(["Day", "Week", "Month"], 1, move |i| p2.set(i)).build(&mut tree);

    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::Items(vec!["Day".into(), "Week".into(), "Month".into()])
    }));
    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::FloatValue(1.0)
    }));

    tree.dispatch(&Event::ValueChanged {
        target: id,
        value: 2.0,
    });
    assert_eq!(picked.get(), 2, "segmented reported the picked index");
}

#[test]
fn stepper_emits_range_and_value_and_reports_changes() {
    let (mut tree, log) = harness();
    let last = Rc::new(Cell::new(0.0_f32));
    let l2 = last.clone();
    let id = stepper(2.0, move |v| l2.set(v))
        .range(0.0, 10.0)
        .step(2.0)
        .build(&mut tree);

    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::Range {
            min: 0.0,
            max: 10.0,
            step: 2.0
        }
    }));
    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::FloatValue(2.0)
    }));

    tree.dispatch(&Event::ValueChanged {
        target: id,
        value: 4.0,
    });
    assert!((last.get() - 4.0).abs() < 1e-6, "stepper reported new value");
}

#[test]
fn text_input_emits_initial_value_placeholder_and_reports_edits() {
    let (mut tree, log) = harness();
    let captured = Rc::new(RefCell::new(String::new()));
    let c2 = captured.clone();
    let id = text_input("hi", move |s| *c2.borrow_mut() = s)
        .placeholder("Name")
        .build(&mut tree);

    {
        let muts = log.borrow();
        assert!(muts.contains(&Mutation::SetAttribute {
            id,
            attr: Attribute::Text("hi".into())
        }));
        assert!(muts.contains(&Mutation::SetAttribute {
            id,
            attr: Attribute::Placeholder("Name".into())
        }));
    }

    tree.dispatch(&Event::TextChanged {
        target: id,
        value: "hello".into(),
        selection: TextSelection::caret(5),
    });
    assert_eq!(&*captured.borrow(), "hello");
}

#[test]
fn image_sets_source() {
    let (mut tree, log) = harness();
    let id = image("star.fill").build(&mut tree);
    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::ImageSource("star.fill".into())
    }));
}

#[test]
fn text_styling_emits_weight_italic_align() {
    use rax_view::{text, TextAlign};
    let (mut tree, log) = harness();
    let id = text("Title")
        .bold()
        .italic()
        .align(TextAlign::Center)
        .build(&mut tree);
    let muts = log.borrow();
    let has = |a: Attribute| muts.contains(&Mutation::SetAttribute { id, attr: a });
    assert!(has(Attribute::FontWeight(700.0)));
    assert!(has(Attribute::Italic(true)));
    assert!(has(Attribute::TextAlign(rax_dom::TextAlign::Center)));
}

#[test]
fn indicators_create_and_progress_sets_value() {
    use rax_dom::WidgetKind;
    use rax_view::{activity_indicator, progress};
    let (mut tree, log) = harness();
    let spinner = activity_indicator().build(&mut tree);
    let bar = progress(0.4).build(&mut tree);
    let muts = log.borrow();
    assert!(muts.contains(&Mutation::Create {
        id: spinner,
        kind: WidgetKind::ActivityIndicator
    }));
    assert!(muts.contains(&Mutation::Create {
        id: bar,
        kind: WidgetKind::Progress
    }));
    assert!(muts.contains(&Mutation::SetAttribute {
        id: bar,
        attr: Attribute::FloatValue(0.4)
    }));
}
