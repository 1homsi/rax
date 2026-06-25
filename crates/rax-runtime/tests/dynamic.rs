//! Dynamic structure (the tab-switch / conditional-content mechanism), verified
//! host-side through the recording backend.

use rax_core::Size;
use rax_dom::{Attribute, Host, Mutation, RecordingBackend};
use rax_reactive::{create_signal, Signal};
use rax_runtime::App;
use rax_view::{boxed, dynamic, text, View};

fn switcher(tab: Signal<u32>) -> impl View {
    dynamic(move || match tab.get() {
        0 => boxed(text("screen zero")),
        _ => boxed(text("screen one")),
    })
}

fn has_text(log: &[Mutation], wanted: &str) -> bool {
    log.iter().any(
        |m| matches!(m, Mutation::SetAttribute { attr: Attribute::Text(s), .. } if s == wanted),
    )
}

#[test]
fn dynamic_content_swaps_when_its_signal_changes() {
    let backend = RecordingBackend::new();
    let log = backend.log();
    let tab = create_signal(0u32);

    let mut app = App::new(Host::new(backend), Size::new(320.0, 640.0), switcher(tab));

    // Initial dynamic build shows screen zero.
    assert!(
        has_text(&log.borrow(), "screen zero"),
        "initial dynamic content built"
    );
    assert!(!has_text(&log.borrow(), "screen one"));

    log.borrow_mut().clear();

    // Switching the signal rebuilds the subtree on the next frame.
    tab.set(1);
    app.tick();

    let muts = log.borrow();
    assert!(has_text(&muts, "screen one"), "new branch built");
    assert!(
        muts.iter().any(|m| matches!(m, Mutation::Destroy { .. })),
        "old branch torn down"
    );
}

#[test]
fn dynamic_list_grows_when_items_are_added() {
    // A reactive list: the dynamic subtree rebuilds to reflect a Vec signal.
    let backend = RecordingBackend::new();
    let log = backend.log();
    let items = create_signal(vec!["a".to_string()]);

    let view = {
        dynamic(move || {
            let current = items.get();
            // Build a text per item (a tiny "list"). One boxed view per render.
            let joined = current.join(",");
            boxed(text(move || joined.clone()))
        })
    };

    let mut app = App::new(Host::new(backend), Size::new(320.0, 640.0), view);
    assert!(has_text(&log.borrow(), "a"));

    log.borrow_mut().clear();
    items.update(|v| v.push("b".to_string()));
    app.tick();

    assert!(
        has_text(&log.borrow(), "a,b"),
        "list rebuilt with the new item"
    );
}
