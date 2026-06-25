//! The `View` trait and `ViewSequence` (heterogeneous tuple children).
//!
//! # The model
//!
//! With fine-grained reactivity, a view's structure is built **once**: dynamic
//! *values* update through signal bindings (one mutation each), and dynamic
//! *structure* (lists/conditionals) is handled by dedicated views that add and
//! remove children via effects. So `View::build` consumes the view, materializes
//! its widget(s) into the [`Tree`], and returns the root [`WidgetId`] — there is
//! no per-frame diff for static trees.

use rax_dom::{Tree, WidgetId};

/// A piece of UI that can be built into the element tree.
pub trait View {
    /// Materializes this view into `tree`, returning the id of the widget it
    /// created. Containers build their children and append them.
    fn build(self, tree: &mut Tree) -> WidgetId;
}

/// A (possibly heterogeneous) sequence of child views, written as a tuple.
///
/// This is the macro-free children syntax: `column((a, b, c))`. Implemented for
/// the empty tuple and tuples up to arity 12 via the internal macro below — the
/// same technique `std` uses for trait impls over tuples. A single child is
/// written `(child,)`.
pub trait ViewSequence {
    /// Builds every child into `parent`, in order.
    fn build_into(self, tree: &mut Tree, parent: WidgetId);
}

impl ViewSequence for () {
    fn build_into(self, _tree: &mut Tree, _parent: WidgetId) {}
}

macro_rules! impl_view_sequence_for_tuple {
    ($($name:ident),+) => {
        impl<$($name: View),+> ViewSequence for ($($name,)+) {
            fn build_into(self, tree: &mut Tree, parent: WidgetId) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(
                    let child = $name.build(tree);
                    tree.append(parent, child);
                )+
            }
        }
    };
}

impl_view_sequence_for_tuple!(A);
impl_view_sequence_for_tuple!(A, B);
impl_view_sequence_for_tuple!(A, B, C);
impl_view_sequence_for_tuple!(A, B, C, D);
impl_view_sequence_for_tuple!(A, B, C, D, E);
impl_view_sequence_for_tuple!(A, B, C, D, E, F);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
