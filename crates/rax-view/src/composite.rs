//! Composable components built **entirely from the public view API**.
//!
//! UIKit has no native checkbox or radio button, so — rather than add engine
//! support — we compose them from [`icon`], [`text`], [`row`], [`dynamic`], and
//! [`ViewExt::on_tap`], exactly as a third-party author would. They double as a
//! worked example: everything here uses only `rax_view`'s public surface, so any
//! consumer can build their own reusable components the same way.
//!
//! Each takes a reactive `state` getter (a `Fn() -> bool`, e.g. a closure over a
//! signal) so the glyph re-renders when the underlying value changes, and an
//! `on_change`/`on_select` callback — the same value-in / event-out shape as the
//! native [`switch`](crate::switch) and [`slider`](crate::slider).

use rax_core::{AlignItems, Color};
use rax_dom::{Tree, WidgetId};

use crate::container::row;
use crate::dynamic::dynamic;
use crate::image::icon;
use crate::modifier::ViewExt;
use crate::text::text;
use crate::view::{boxed, BoxedView, View};

/// The default accent used for a checked/selected glyph (iOS system blue).
const DEFAULT_TINT: Color = Color::rgb(0, 122, 255);
const GLYPH: f32 = 24.0;

/// A labelled checkbox. Build via [`checkbox`].
pub struct Checkbox<S, F> {
    checked: S,
    label: Option<String>,
    on_change: F,
    tint: Color,
}

/// Creates a checkbox whose checked state is read from `checked` (re-read
/// reactively, so it updates when the underlying value changes) and that calls
/// `on_change` with the toggled value when tapped.
///
/// ```
/// use rax_view::checkbox;
/// use rax_reactive::create_signal;
///
/// let agreed = create_signal(false);
/// let view = checkbox(move || agreed.get(), move |v| agreed.set(v))
///     .label("I agree to the terms");
/// ```
pub fn checkbox<S, F>(checked: S, on_change: F) -> Checkbox<S, F>
where
    S: Fn() -> bool + Clone + 'static,
    F: FnMut(bool) + 'static,
{
    Checkbox {
        checked,
        label: None,
        on_change,
        tint: DEFAULT_TINT,
    }
}

impl<S, F> Checkbox<S, F> {
    /// Adds a trailing text label (also part of the tap target).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Overrides the accent color of the checked glyph.
    #[must_use]
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }
}

impl<S, F> View for Checkbox<S, F>
where
    S: Fn() -> bool + Clone + 'static,
    F: FnMut(bool) + 'static,
{
    fn build(self, tree: &mut Tree) -> WidgetId {
        let tint = self.tint;
        let checked_for_glyph = self.checked.clone();
        let glyph = dynamic(move || {
            let symbol = if checked_for_glyph() {
                "checkmark.square.fill"
            } else {
                "square"
            };
            boxed(icon(symbol).tint(tint).size(GLYPH, GLYPH))
        })
        .grow(0.0);

        let checked_for_tap = self.checked;
        let mut on_change = self.on_change;
        let toggle = move || on_change(!checked_for_tap());

        let content: BoxedView = match self.label {
            Some(label) => boxed(
                row((glyph, text(label).font_size(16.0)))
                    .gap(10.0)
                    .align(AlignItems::Center),
            ),
            None => boxed(glyph),
        };
        content.on_tap(toggle).build(tree)
    }
}

/// A labelled radio button (one option of a group). Build via [`radio`].
pub struct Radio<S, F> {
    selected: S,
    label: Option<String>,
    on_select: F,
    tint: Color,
}

/// Creates a radio button whose selected state is read from `selected` and that
/// calls `on_select` when tapped. Group several over a shared signal — each
/// `selected` closure compares the signal to its own value, and `on_select`
/// sets the signal — to get single-selection behaviour.
///
/// ```
/// use rax_view::radio;
/// use rax_reactive::create_signal;
///
/// let choice = create_signal(0u32);
/// let first = radio(move || choice.get() == 0, move || choice.set(0)).label("One");
/// let second = radio(move || choice.get() == 1, move || choice.set(1)).label("Two");
/// ```
pub fn radio<S, F>(selected: S, on_select: F) -> Radio<S, F>
where
    S: Fn() -> bool + Clone + 'static,
    F: FnMut() + 'static,
{
    Radio {
        selected,
        label: None,
        on_select,
        tint: DEFAULT_TINT,
    }
}

impl<S, F> Radio<S, F> {
    /// Adds a trailing text label (also part of the tap target).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Overrides the accent color of the selected glyph.
    #[must_use]
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }
}

impl<S, F> View for Radio<S, F>
where
    S: Fn() -> bool + Clone + 'static,
    F: FnMut() + 'static,
{
    fn build(self, tree: &mut Tree) -> WidgetId {
        let tint = self.tint;
        let selected_for_glyph = self.selected;
        let glyph = dynamic(move || {
            let symbol = if selected_for_glyph() {
                "largecircle.fill.circle"
            } else {
                "circle"
            };
            boxed(icon(symbol).tint(tint).size(GLYPH, GLYPH))
        })
        .grow(0.0);

        let mut on_select = self.on_select;
        let select = move || on_select();

        let content: BoxedView = match self.label {
            Some(label) => boxed(
                row((glyph, text(label).font_size(16.0)))
                    .gap(10.0)
                    .align(AlignItems::Center),
            ),
            None => boxed(glyph),
        };
        content.on_tap(select).build(tree)
    }
}
