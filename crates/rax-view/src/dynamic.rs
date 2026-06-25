//! Dynamic structure: a subtree that reactively rebuilds when its inputs change.
//!
//! This is the one place "reconciliation" happens in a fine-grained system —
//! everything else binds values in place. [`dynamic`] takes a selector that
//! reads signals and returns a [`BoxedView`]; whenever those signals change, the
//! subtree is torn down and rebuilt. Use it for tab switching, conditional
//! content, and lists.

use rax_core::{Dimension, LayoutStyle};
use rax_dom::{BuildThunk, Tree, WidgetId};

use crate::view::{BoxedView, View};

/// A reactive subtree. Build via [`dynamic`].
pub struct Dynamic<F> {
    selector: F,
    grow: f32,
}

/// Creates a dynamic subtree driven by `selector`.
///
/// `selector` reads signals and returns the view to show; when those signals
/// change, the subtree rebuilds. By default the container grows to fill its
/// parent (it is usually a content area); use [`grow`](Dynamic::grow) to change.
pub fn dynamic<F>(selector: F) -> Dynamic<F>
where
    F: FnMut() -> BoxedView + 'static,
{
    Dynamic {
        selector,
        grow: 1.0,
    }
}

impl<F: FnMut() -> BoxedView + 'static> Dynamic<F> {
    /// Sets the flex-grow factor of the dynamic container (default `1.0`).
    #[must_use]
    pub fn grow(mut self, factor: f32) -> Self {
        self.grow = factor;
        self
    }
}

impl<F: FnMut() -> BoxedView + 'static> View for Dynamic<F> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let mut selector = self.selector;
        let id = tree.create_dynamic(move || {
            let view = selector();
            let thunk: BuildThunk = Box::new(move |tree| view.build(tree));
            thunk
        });
        tree.set_style(
            id,
            LayoutStyle {
                flex_grow: self.grow,
                width: Dimension::Auto,
                ..LayoutStyle::default()
            },
        );
        id
    }
}
