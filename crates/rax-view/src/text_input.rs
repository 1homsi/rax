//! The `text_input` view: a controlled single-line text field.

use rax_core::Color;
use rax_dom::{Attribute, Event, EventKind, Tree, WidgetId};

use crate::view::View;

/// A single-line text field. Build via [`text_input`].
pub struct TextInput<F> {
    value: String,
    placeholder: Option<String>,
    color: Option<Color>,
    on_change: F,
}

/// Creates a text field with initial `value` that calls `on_change` with the
/// full text on every edit.
pub fn text_input<F: FnMut(String) + 'static>(
    value: impl Into<String>,
    on_change: F,
) -> TextInput<F> {
    TextInput {
        value: value.into(),
        placeholder: None,
        color: None,
        on_change,
    }
}

impl<F: FnMut(String) + 'static> TextInput<F> {
    /// Sets placeholder text shown when empty.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Sets the text color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl<F: FnMut(String) + 'static> View for TextInput<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_text_input();
        tree.set(id, Attribute::Text(self.value));
        if let Some(placeholder) = self.placeholder {
            tree.set(id, Attribute::Placeholder(placeholder));
        }
        if let Some(color) = self.color {
            tree.set(id, Attribute::TextColor(color));
        }
        let mut on_change = self.on_change;
        tree.on(id, EventKind::TextChanged, move |event| {
            if let Event::TextChanged { value, .. } = event {
                on_change(value.clone());
            }
        });
        id
    }
}
