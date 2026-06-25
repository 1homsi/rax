//! Flex containers: `column` (vertical) and `row` (horizontal), with layout and
//! paint modifiers. Children are a [`ViewSequence`] (a tuple).

use rax_core::{Color, EdgeInsets};
use rax_dom::{Attribute, Axis, Tree, WidgetId};

use crate::view::{View, ViewSequence};

/// A flex container view. Build via [`column`] or [`row`].
pub struct Container<C: ViewSequence> {
    axis: Axis,
    children: C,
    padding: Option<EdgeInsets>,
    gap: Option<f32>,
    background: Option<Color>,
}

fn container<C: ViewSequence>(axis: Axis, children: C) -> Container<C> {
    Container {
        axis,
        children,
        padding: None,
        gap: None,
        background: None,
    }
}

/// A vertically-stacked container.
pub fn column<C: ViewSequence>(children: C) -> Container<C> {
    container(Axis::Vertical, children)
}

/// A horizontally-stacked container.
pub fn row<C: ViewSequence>(children: C) -> Container<C> {
    container(Axis::Horizontal, children)
}

impl<C: ViewSequence> Container<C> {
    /// Uniform padding on all edges.
    #[must_use]
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Some(EdgeInsets::all(value));
        self
    }

    /// Explicit per-edge padding.
    #[must_use]
    pub fn padding_insets(mut self, insets: EdgeInsets) -> Self {
        self.padding = Some(insets);
        self
    }

    /// Spacing between children along the primary axis.
    #[must_use]
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = Some(value);
        self
    }

    /// Background fill color.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl<C: ViewSequence> View for Container<C> {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_view();
        tree.set(id, Attribute::FlexDirection(self.axis));
        if let Some(padding) = self.padding {
            tree.set(id, Attribute::Padding(padding));
        }
        if let Some(gap) = self.gap {
            tree.set(id, Attribute::Gap(gap));
        }
        if let Some(background) = self.background {
            tree.set(id, Attribute::BackgroundColor(background));
        }
        self.children.build_into(tree, id);
        id
    }
}
