//! The `text` view, accepting either a static string or a reactive closure.
//!
//! `text("hi")` and `text(move || format!("{}", n.get()))` both work. Overloading
//! on "value vs closure" in Rust hits coherence limits, so we use the marker-type
//! trick (a phantom `M` distinguishes the impls) — the same approach axum and
//! Bevy use for "accept a value or a function".

use core::marker::PhantomData;

use rax_core::Color;
use rax_dom::{Attribute, Tree, WidgetId};

use crate::view::View;

/// Marker: the text is a fixed string set once.
pub struct StaticText;
/// Marker: the text is a reactive closure bound to signals.
pub struct DynamicText;

/// Converts an argument to `text` into a binding on a text widget.
pub trait IntoText<M> {
    /// Applies the value to `id` in `tree` (static set or reactive bind).
    fn apply(self, tree: &mut Tree, id: WidgetId);
}

impl IntoText<StaticText> for &str {
    fn apply(self, tree: &mut Tree, id: WidgetId) {
        tree.set(id, Attribute::Text(self.to_string()));
    }
}

impl IntoText<StaticText> for String {
    fn apply(self, tree: &mut Tree, id: WidgetId) {
        tree.set(id, Attribute::Text(self));
    }
}

impl<F: FnMut() -> String + 'static> IntoText<DynamicText> for F {
    fn apply(self, tree: &mut Tree, id: WidgetId) {
        let mut f = self;
        tree.bind(id, move || Attribute::Text(f()));
    }
}

/// A text label view. Build via [`text`].
pub struct Text<M, T: IntoText<M>> {
    value: T,
    font_size: Option<f32>,
    color: Option<Color>,
    _marker: PhantomData<fn() -> M>,
}

/// Creates a text view from a static string or a reactive `FnMut() -> String`.
pub fn text<M, T: IntoText<M>>(value: T) -> Text<M, T> {
    Text {
        value,
        font_size: None,
        color: None,
        _marker: PhantomData,
    }
}

impl<M, T: IntoText<M>> Text<M, T> {
    /// Sets the font size in logical pixels.
    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Sets the text color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl<M, T: IntoText<M>> View for Text<M, T> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_text();
        if let Some(size) = self.font_size {
            tree.set(id, Attribute::FontSize(size));
        }
        if let Some(color) = self.color {
            tree.set(id, Attribute::TextColor(color));
        }
        self.value.apply(tree, id);
        id
    }
}
