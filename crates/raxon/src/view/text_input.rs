//! The `text_input` view: a controlled single-line text field.

use crate::core::Color;
use crate::dom::{Attribute, Event, EventKind, KeyboardType, ReturnKeyType, Tree, WidgetId};


use super::view::View;

/// A single-line text field. Build via [`text_input`].
pub struct TextInput<F> {
    value: String,
    placeholder: Option<String>,
    placeholder_color: Option<Color>,
    color: Option<Color>,
    on_change: F,
    return_key: Option<ReturnKeyType>,
    keyboard_type: Option<KeyboardType>,
    secure: bool,
    on_submit: Option<Box<dyn FnMut()>>,
    prefix: Option<String>,
    suffix: Option<String>,
    clear_button: bool,
    read_only: bool,
    max_length: Option<usize>,
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
        placeholder_color: None,
        color: None,
        on_change,
        return_key: None,
        keyboard_type: None,
        secure: false,
        on_submit: None,
        prefix: None,
        suffix: None,
        clear_button: false,
        read_only: false,
        max_length: None,
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

    /// Sets the return key label shown on the keyboard.
    #[must_use]
    pub fn return_key(mut self, key: ReturnKeyType) -> Self {
        self.return_key = Some(key);
        self
    }

    /// Sets the keyboard type (e.g. email, number pad, phone).
    #[must_use]
    pub fn keyboard_type(mut self, kt: KeyboardType) -> Self {
        self.keyboard_type = Some(kt);
        self
    }

    /// Makes this a secure (password) input field.
    #[must_use]
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Called when the user presses the return/submit key.
    #[must_use]
    pub fn on_submit(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Sets the color of the placeholder text.
    #[must_use]
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    /// Prepends a short, non-editable label to the left of the input.
    #[must_use]
    pub fn prefix(mut self, text: impl Into<String>) -> Self {
        self.prefix = Some(text.into());
        self
    }

    /// Appends a short, non-editable label to the right of the input.
    #[must_use]
    pub fn suffix(mut self, text: impl Into<String>) -> Self {
        self.suffix = Some(text.into());
        self
    }

    /// Shows or hides the built-in clear (×) button inside the text field.
    #[must_use]
    pub fn clear_button(mut self, show: bool) -> Self {
        self.clear_button = show;
        self
    }

    /// Makes the field read-only (non-editable). The user can still select and
    /// copy text, but cannot type new characters.
    #[must_use]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Limits input to `n` characters. The platform delegate will reject any
    /// characters beyond this count.
    #[must_use]
    pub fn max_length(mut self, n: usize) -> Self {
        self.max_length = Some(n);
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
        if let Some(key) = self.return_key {
            tree.set(id, Attribute::ReturnKey(key));
        }
        if let Some(kt) = self.keyboard_type {
            tree.set(id, Attribute::KeyboardType(kt));
        }
        if self.secure {
            tree.set(id, Attribute::Secure(true));
        }
        if let Some(color) = self.placeholder_color {
            tree.set(id, Attribute::PlaceholderColor(color));
        }
        if let Some(p) = self.prefix {
            tree.set(id, Attribute::InputPrefix(p));
        }
        if let Some(s) = self.suffix {
            tree.set(id, Attribute::InputSuffix(s));
        }
        if self.clear_button {
            tree.set(id, Attribute::ClearButton(true));
        }
        if self.read_only {
            tree.set(id, Attribute::ReadOnly(true));
        }
        if let Some(n) = self.max_length {
            tree.set(id, Attribute::MaxLength(n));
        }
        let mut on_change = self.on_change;
        tree.on(id, EventKind::TextChanged, move |event| {
            if let Event::TextChanged { value, .. } = event {
                on_change(value.clone());
            }
        });
        if let Some(mut on_submit) = self.on_submit {
            tree.on(id, EventKind::Submit, move |event| {
                if matches!(event, Event::Submit { .. }) {
                    on_submit();
                }
            });
        }
        id
    }
}

/// A multi-line text area. Build via [`text_area`].
pub struct TextArea<F> {
    value: String,
    placeholder: Option<String>,
    color: Option<Color>,
    on_change: F,
}

/// Creates a multi-line text area with initial `value` that calls `on_change` on
/// every edit.
pub fn text_area<F: FnMut(String) + 'static>(
    value: impl Into<String>,
    on_change: F,
) -> TextArea<F> {
    TextArea {
        value: value.into(),
        placeholder: None,
        color: None,
        on_change,
    }
}

impl<F: FnMut(String) + 'static> TextArea<F> {
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

impl<F: FnMut(String) + 'static> View for TextArea<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_text_area();
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
