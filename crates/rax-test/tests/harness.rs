//! The test harness, testing itself against small apps.

use rax_dom::WidgetKind;
use rax_reactive::{create_signal, Signal};
use rax_test::TestHarness;
use rax_view::{boxed, button, column, dynamic, slider, switch, text, View};

fn counter(count: Signal<i32>) -> impl View {
    column((
        text(move || format!("Count: {}", count.get())),
        button("Increment", move || count.update(|c| *c += 1)),
    ))
}

#[test]
fn find_and_tap_drives_reactivity() {
    let count = create_signal(0);
    let mut ui = TestHarness::mount(move || counter(count));

    assert!(ui.find_text("Count: 0").is_some());
    let btn = ui.find_button("Increment").expect("button by title");

    ui.tap(btn);
    ui.assert_text("Count: 1");
    ui.tap(btn);
    ui.assert_text("Count: 2");
}

#[test]
fn widgets_of_kind_counts_controls() {
    let on = create_signal(false);
    let view = column((
        switch(false, move |v| on.set(v)),
        slider(0.5, |_| {}),
        text("hi"),
    ));
    let ui = TestHarness::mount(move || view);
    assert_eq!(ui.widgets_of_kind(WidgetKind::Switch).len(), 1);
    assert_eq!(ui.widgets_of_kind(WidgetKind::Slider).len(), 1);
    assert_eq!(ui.widgets_of_kind(WidgetKind::Text).len(), 1);
}

#[test]
fn set_value_reports_through_to_state() {
    let value = create_signal(0.0_f32);
    let v2 = value;
    let view = column((slider(0.0, move |v| v2.set(v)),));
    let mut ui = TestHarness::mount(move || view);
    let s = ui.widgets_of_kind(WidgetKind::Slider)[0];
    ui.set_value(s, 0.75);
    assert!((value.get() - 0.75).abs() < 1e-6);
}

#[test]
fn dynamic_list_updates_are_queryable() {
    let items = create_signal(vec!["a".to_string()]);
    let view = dynamic(move || {
        let joined = items.get().join(",");
        boxed(text(move || joined.clone()))
    });
    let mut ui = TestHarness::mount(move || view);
    assert!(ui.find_text("a").is_some());

    items.update(|v| v.push("b".into()));
    ui.tick();
    assert!(ui.find_text("a,b").is_some());
}
