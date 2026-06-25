//! The `scroll` view: a scrolling container (vertical or horizontal),
//! optionally with pull-to-refresh.

use rax_core::{FlexDirection, LayoutStyle};
use rax_dom::{Attribute, Event, EventKind, Tree, WidgetId};

use crate::view::View;

/// A scroll container wrapping a single child (usually a column or row).
/// Build via [`scroll`].
pub struct Scroll<V> {
    child: V,
    grow: f32,
    horizontal: bool,
    refreshing: Option<bool>,
    on_refresh: Option<Box<dyn FnMut()>>,
}

/// Wraps `child` in a vertically-scrolling container that fills its parent.
pub fn scroll<V: View>(child: V) -> Scroll<V> {
    Scroll {
        child,
        grow: 1.0,
        horizontal: false,
        refreshing: None,
        on_refresh: None,
    }
}

impl<V: View> Scroll<V> {
    /// Sets the flex-grow factor of the scroll container (default `1.0`).
    #[must_use]
    pub fn grow(mut self, factor: f32) -> Self {
        self.grow = factor;
        self
    }

    /// Makes this a horizontal scroll view (content lays out in a row).
    #[must_use]
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Enables pull-to-refresh. `is_refreshing` controls the spinner visibility;
    /// `on_refresh` is called when the user pulls to refresh.
    #[must_use]
    pub fn refreshable(mut self, is_refreshing: bool, on_refresh: impl FnMut() + 'static) -> Self {
        self.refreshing = Some(is_refreshing);
        self.on_refresh = Some(Box::new(on_refresh));
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
                direction: if self.horizontal {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                ..LayoutStyle::default()
            },
        );
        if self.horizontal {
            tree.set(id, Attribute::Horizontal(true));
        }
        if let Some(refreshing) = self.refreshing {
            tree.set(id, Attribute::Refreshing(refreshing));
            if let Some(mut on_refresh) = self.on_refresh {
                tree.on(id, EventKind::Refresh, move |event| {
                    if matches!(event, Event::Refresh { .. }) {
                        on_refresh();
                    }
                });
            }
        }
        let child = self.child.build(tree);
        tree.append(id, child);
        id
    }
}
