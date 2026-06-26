//! A flexible empty view that absorbs free space along the parent's main axis.

use crate::core::LayoutStyle;
use crate::dom::{Tree, WidgetId};

use super::view::View;

/// A flexible gap. Build via [`spacer`].
pub struct Spacer {
    grow: f32,
}

/// Creates a spacer that expands to fill available space (flex-grow `1.0`).
pub fn spacer() -> Spacer {
    Spacer { grow: 1.0 }
}

impl Spacer {
    /// Sets the flex-grow factor.
    #[must_use]
    pub fn grow(mut self, factor: f32) -> Self {
        self.grow = factor;
        self
    }
}

impl View for Spacer {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_view();
        tree.set_style(
            id,
            LayoutStyle {
                flex_grow: self.grow,
                ..LayoutStyle::default()
            },
        );
        id
    }
}
