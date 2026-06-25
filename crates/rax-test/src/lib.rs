//! Headless testing for `rax` apps — no simulator, no device.
//!
//! [`TestHarness`] mounts a view against a recording backend, then lets tests
//! *query* the resulting UI (find by text or kind, read a widget's current text)
//! and *interact* with it (tap, change a control's value), driving a frame after
//! each interaction so reactive updates settle — exactly like a user would.
//!
//! ```
//! use rax_test::TestHarness;
//! use rax_view::{button, column, text, View};
//! use rax_reactive::{create_signal, Signal};
//!
//! fn counter(count: Signal<i32>) -> impl View {
//!     column((
//!         text(move || format!("Count: {}", count.get())),
//!         button("inc", move || count.update(|c| *c += 1)),
//!     ))
//! }
//!
//! let count = create_signal(0);
//! let mut ui = TestHarness::mount(counter(count));
//! assert!(ui.find_text("Count: 0").is_some());
//! let btn = ui.find_button("inc").unwrap();
//! ui.tap(btn);
//! assert!(ui.find_text("Count: 1").is_some());
//! ```

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use rax_core::Size;
use rax_dom::{Attribute, Event, Host, Mutation, RecordingBackend, WidgetId, WidgetKind};
use rax_runtime::App;
use rax_view::View;

/// A mounted app under test, with query + interaction helpers.
pub struct TestHarness {
    app: App,
    log: Rc<RefCell<Vec<Mutation>>>,
}

impl TestHarness {
    /// Mounts `view` in a default 390×844 viewport (iPhone-ish).
    pub fn mount(view: impl View) -> TestHarness {
        TestHarness::mount_sized(view, Size::new(390.0, 844.0))
    }

    /// Mounts `view` at an explicit viewport size.
    pub fn mount_sized(view: impl View, viewport: Size) -> TestHarness {
        let backend = RecordingBackend::new();
        let log = backend.log();
        let app = App::new(Host::new(backend), viewport, view);
        TestHarness { app, log }
    }

    /// Advances one frame (drains events, re-runs dynamics, re-lays-out).
    pub fn tick(&mut self) {
        self.app.tick();
    }

    /// A snapshot of all mutations emitted so far.
    pub fn mutations(&self) -> Vec<Mutation> {
        self.log.borrow().clone()
    }

    /// The current (most recently set) text of a widget, if it has any.
    pub fn text_of(&self, id: WidgetId) -> Option<String> {
        self.log.borrow().iter().rev().find_map(|m| match m {
            Mutation::SetAttribute {
                id: i,
                attr: Attribute::Text(s),
            } if *i == id => Some(s.clone()),
            _ => None,
        })
    }

    /// Finds the first widget whose current text exactly equals `text`.
    pub fn find_text(&self, text: &str) -> Option<WidgetId> {
        self.find_text_where(|s| s == text)
    }

    /// Finds the first widget whose current text contains `substring`.
    pub fn find_text_containing(&self, substring: &str) -> Option<WidgetId> {
        self.find_text_where(|s| s.contains(substring))
    }

    fn find_text_where(&self, pred: impl Fn(&str) -> bool) -> Option<WidgetId> {
        // Collect each widget's latest text, then test the predicate.
        let log = self.log.borrow();
        let mut latest: Vec<(WidgetId, String)> = Vec::new();
        for m in log.iter() {
            if let Mutation::SetAttribute {
                id,
                attr: Attribute::Text(s),
            } = m
            {
                if let Some(slot) = latest.iter_mut().find(|(i, _)| i == id) {
                    slot.1 = s.clone();
                } else {
                    latest.push((*id, s.clone()));
                }
            }
        }
        latest.into_iter().find(|(_, s)| pred(s)).map(|(id, _)| id)
    }

    /// Finds a button by its (current) title.
    pub fn find_button(&self, title: &str) -> Option<WidgetId> {
        let id = self.find_text(title)?;
        if self.kind_of(id) == Some(WidgetKind::Button) {
            Some(id)
        } else {
            None
        }
    }

    /// All widget ids created with the given kind, in creation order.
    pub fn widgets_of_kind(&self, kind: WidgetKind) -> Vec<WidgetId> {
        self.log
            .borrow()
            .iter()
            .filter_map(|m| match m {
                Mutation::Create { id, kind: k } if *k == kind => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// The kind a widget was created as.
    pub fn kind_of(&self, id: WidgetId) -> Option<WidgetKind> {
        self.log.borrow().iter().find_map(|m| match m {
            Mutation::Create { id: i, kind } if *i == id => Some(*kind),
            _ => None,
        })
    }

    /// Simulates a tap on `id` and advances a frame.
    pub fn tap(&mut self, id: WidgetId) {
        self.app.event_sink().dispatch(Event::Tap { target: id });
        self.tick();
    }

    /// Simulates a control value change and advances a frame.
    pub fn set_value(&mut self, id: WidgetId, value: f64) {
        self.app
            .event_sink()
            .dispatch(Event::ValueChanged { target: id, value });
        self.tick();
    }

    /// Dispatches an arbitrary event and advances a frame.
    pub fn dispatch(&mut self, event: Event) {
        self.app.event_sink().dispatch(event);
        self.tick();
    }

    /// Asserts that some widget currently shows `text` (panics otherwise).
    pub fn assert_text(&self, text: &str) {
        assert!(
            self.find_text(text).is_some(),
            "expected a widget showing {text:?}"
        );
    }
}
