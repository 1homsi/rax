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

/// Object-safe shim behind [`BoxedView`] (`View::build` takes `self` by value,
/// which is not object-safe; this takes `Box<Self>`).
trait AnyView {
    fn build_boxed(self: Box<Self>, tree: &mut Tree) -> WidgetId;
}

impl<V: View> AnyView for V {
    fn build_boxed(self: Box<Self>, tree: &mut Tree) -> WidgetId {
        (*self).build(tree)
    }
}

/// A type-erased view. Lets heterogeneous branches (e.g. different screens per
/// tab) share one type, which dynamic views require.
pub struct BoxedView(Box<dyn AnyView>);

/// Erases a view's concrete type.
pub fn boxed<V: View + 'static>(view: V) -> BoxedView {
    BoxedView(Box::new(view))
}

impl View for BoxedView {
    fn build(self, tree: &mut Tree) -> WidgetId {
        self.0.build_boxed(tree)
    }
}

/// A (possibly heterogeneous) sequence of child views, written as a tuple.
///
/// This is the macro-free children syntax: `column((a, b, c))`. Implemented for
/// the empty tuple and tuples up to arity 16 via the internal macro below — the
/// same technique `std` uses for trait impls over tuples. A single child is
/// written `(child,)`. For more children than that — or a dynamic count — pass a
/// `Vec<BoxedView>` (see [`boxed`]) instead.
pub trait ViewSequence {
    /// Builds every child into `parent`, in order.
    fn build_into(self, tree: &mut Tree, parent: WidgetId);
}

impl ViewSequence for () {
    fn build_into(self, _tree: &mut Tree, _parent: WidgetId) {}
}

/// A runtime-sized list of children — the basis for rendering collections
/// (`column(items.into_iter().map(boxed).collect::<Vec<_>>())`).
impl ViewSequence for Vec<BoxedView> {
    fn build_into(self, tree: &mut Tree, parent: WidgetId) {
        for view in self {
            let child = view.build(tree);
            tree.append(parent, child);
        }
    }
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
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_sequence_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
