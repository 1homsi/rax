//! Value controls: `switch`, `slider`, `segmented`, and `stepper`.

use raxon_dom::{Attribute, Event, EventKind, Tree, WidgetId};

use crate::view::View;

/// An on/off switch. Build via [`switch`].
pub struct Switch<F> {
    checked: bool,
    on_change: F,
}

/// Creates a switch with initial state `checked` that calls `on_change` when
/// toggled.
pub fn switch<F: FnMut(bool) + 'static>(checked: bool, on_change: F) -> Switch<F> {
    Switch { checked, on_change }
}

impl<F: FnMut(bool) + 'static> View for Switch<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_switch();
        tree.set(id, Attribute::BoolValue(self.checked));
        let mut on_change = self.on_change;
        tree.on(id, EventKind::ValueChanged, move |event| {
            if let Event::ValueChanged { value, .. } = event {
                on_change(*value != 0.0);
            }
        });
        id
    }
}

/// A value slider (`0.0..=1.0`). Build via [`slider`].
pub struct Slider<F> {
    value: f32,
    on_change: F,
}

/// Creates a slider at `value` (`0.0..=1.0`) that calls `on_change` as it moves.
pub fn slider<F: FnMut(f32) + 'static>(value: f32, on_change: F) -> Slider<F> {
    Slider { value, on_change }
}

impl<F: FnMut(f32) + 'static> View for Slider<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_slider();
        tree.set(id, Attribute::FloatValue(self.value));
        let mut on_change = self.on_change;
        tree.on(id, EventKind::ValueChanged, move |event| {
            if let Event::ValueChanged { value, .. } = event {
                on_change(*value as f32);
            }
        });
        id
    }
}

/// A horizontal segmented control (pick one of N labelled options). Build via
/// [`segmented`].
pub struct Segmented<F> {
    items: Vec<String>,
    selected: usize,
    on_change: F,
}

/// Creates a segmented control over `items`, with `selected` initially active,
/// calling `on_change` with the newly selected index when the user picks a
/// segment.
pub fn segmented<F>(
    items: impl IntoIterator<Item = impl Into<String>>,
    selected: usize,
    on_change: F,
) -> Segmented<F>
where
    F: FnMut(usize) + 'static,
{
    Segmented {
        items: items.into_iter().map(Into::into).collect(),
        selected,
        on_change,
    }
}

impl<F: FnMut(usize) + 'static> View for Segmented<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_segmented();
        tree.set(id, Attribute::Items(self.items));
        tree.set(id, Attribute::FloatValue(self.selected as f32));
        let mut on_change = self.on_change;
        tree.on(id, EventKind::ValueChanged, move |event| {
            if let Event::ValueChanged { value, .. } = event {
                on_change(value.max(0.0) as usize);
            }
        });
        id
    }
}

/// A -/+ stepper over a bounded numeric range. Build via [`stepper`].
pub struct Stepper<F> {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    on_change: F,
}

/// Creates a stepper at `value`, reporting the new value via `on_change` when
/// the user taps -/+. Defaults to a `0..=100` range with a step of `1`; tune
/// with [`Stepper::range`] and [`Stepper::step`].
pub fn stepper<F: FnMut(f32) + 'static>(value: f32, on_change: F) -> Stepper<F> {
    Stepper {
        value,
        min: 0.0,
        max: 100.0,
        step: 1.0,
        on_change,
    }
}

impl<F> Stepper<F> {
    /// Sets the inclusive `min..=max` bounds.
    #[must_use]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Sets the increment applied per -/+ tap.
    #[must_use]
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }
}

impl<F: FnMut(f32) + 'static> View for Stepper<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_stepper();
        tree.set(
            id,
            Attribute::Range {
                min: self.min,
                max: self.max,
                step: self.step,
            },
        );
        tree.set(id, Attribute::FloatValue(self.value));
        let mut on_change = self.on_change;
        tree.on(id, EventKind::ValueChanged, move |event| {
            if let Event::ValueChanged { value, .. } = event {
                on_change(*value as f32);
            }
        });
        id
    }
}
