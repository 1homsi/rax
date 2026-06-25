//! The `scroll` view: a vertically-scrolling container.

use rax_core::LayoutStyle;
use rax_dom::{Tree, WidgetId};

use crate::view::View;

/// A vertical scroll container wrapping a single child (usually a column).
/// Build via [`scroll`].
pub struct Scroll<V> {
    child: V,
    grow: f32,
}

/// Wraps `child` in a vertically-scrolling container that fills its parent.
pub fn scroll<V: View>(child: V) -> Scroll<V> {
    Scroll { child, grow: 1.0 }
}

impl<V: View> Scroll<V> {
    /// Sets the flex-grow factor of the scroll container (default `1.0`).
    #[must_use]
    pub fn grow(mut self, factor: f32) -> Self {
        self.grow = factor;
        self
    }
}

impl<V: View> View for Scroll<V> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_scroll();
        tree.set_style(
            id,
            LayoutStyle {
                scroll: true,
                flex_grow: self.grow,
                ..LayoutStyle::default()
            },
        );
        let child = self.child.build(tree);
        tree.append(id, child);
        id
    }
}
