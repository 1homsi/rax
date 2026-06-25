//! The `button` view: a tappable widget with a label and a tap handler.

use rax_dom::{Attribute, EventKind, Tree, WidgetId};

use crate::view::View;

/// A button view. Build via [`button`].
pub struct Button<F> {
    label: String,
    on_tap: F,
}

/// Creates a button with `label` that runs `on_tap` when tapped.
pub fn button<F: FnMut() + 'static>(label: impl Into<String>, on_tap: F) -> Button<F> {
    Button {
        label: label.into(),
        on_tap,
    }
}

impl<F: FnMut() + 'static> View for Button<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_button();
        tree.set(id, Attribute::Text(self.label));
        let mut on_tap = self.on_tap;
        tree.on(id, EventKind::Tap, move |_event| on_tap());
        id
    }
}
