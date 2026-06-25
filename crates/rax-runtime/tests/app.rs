//! Full runtime pipeline, host-side: build → layout → tap → reactive update,
//! all observed through the recording backend (no platform needed).

use rax_core::Size;
use rax_dom::{Attribute, Event, Host, Mutation, RecordingBackend, WidgetId, WidgetKind};
use rax_reactive::{create_signal, Signal};
use rax_runtime::App;
use rax_view::{button, column, text, View};

fn counter(count: Signal<i32>) -> impl View {
    column((
        text(move || format!("Count: {}", count.get())),
        button("+1", move || count.update(|c| *c += 1)),
    ))
    .padding(16.0)
    .gap(8.0)
}

fn find_button(log: &[Mutation]) -> WidgetId {
    log.iter()
        .find_map(|m| match m {
            Mutation::Create {
                id,
                kind: WidgetKind::Button,
            } => Some(*id),
            _ => None,
        })
        .expect("counter has a button")
}

#[test]
fn app_builds_lays_out_and_reacts_to_taps() {
    let backend = RecordingBackend::new();
    let log = backend.log();
    let count = create_signal(0);

    let mut app = App::new(Host::new(backend), Size::new(320.0, 640.0), move || {
        counter(count)
    });

    // Initial build emitted Create + paint, and the initial layout emitted frames.
    {
        let muts = log.borrow();
        assert!(muts.iter().any(|m| matches!(
            m,
            Mutation::Create {
                kind: WidgetKind::View,
                ..
            }
        )));
        assert!(
            muts.iter().any(|m| matches!(m, Mutation::SetFrame { .. })),
            "initial layout emits frames"
        );
        // The root fills the viewport.
        assert!(muts.iter().any(|m| matches!(
            m,
            Mutation::SetFrame { id, rect } if *id == app.root() && rect.size == Size::new(320.0, 640.0)
        )));
    }

    let button_id = find_button(&log.borrow());
    log.borrow_mut().clear();

    // The platform delivers a tap; the next frame processes it.
    app.event_sink().dispatch(Event::Tap { target: button_id });
    app.tick();

    let muts = log.borrow();
    assert!(
        muts.iter().any(|m| matches!(m, Mutation::SetAttribute { attr: Attribute::Text(s), .. } if s == "Count: 1")),
        "tap incremented the counter and re-rendered the label"
    );
}

#[test]
fn relayout_emits_no_redundant_frames_when_nothing_changes() {
    let backend = RecordingBackend::new();
    let log = backend.log();
    let count = create_signal(0);
    let mut app = App::new(Host::new(backend), Size::new(320.0, 640.0), move || {
        counter(count)
    });

    log.borrow_mut().clear();
    app.tick(); // no events, no size change

    let frame_mutations = log
        .borrow()
        .iter()
        .filter(|m| matches!(m, Mutation::SetFrame { .. }))
        .count();
    assert_eq!(frame_mutations, 0, "stable layout emits no frames");
}
