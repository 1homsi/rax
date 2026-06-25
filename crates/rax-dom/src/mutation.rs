//! The widget/mutation model: the data crossing the render seam.
//!
//! The engine never touches a platform view directly. Instead it produces a
//! stream of [`Mutation`]s — a flat, `Clone`-able, comparable command list — and
//! a [`Backend`](crate::Backend) applies them to real `UIView`s /
//! `android.view.View`s. Keeping this an inspectable value type is what makes
//! the whole framework testable with zero platform code (assert on the stream)
//! and is the seam that later allows diffing off the main thread.

use rax_core::{Color, EdgeInsets, Index};

/// A stable handle to a node in the retained element tree (and, 1:1, to a native
/// view created by the backend).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub(crate) Index);

impl WidgetId {
    /// The raw slot, for debugging / inspector tooling.
    pub fn raw(self) -> u32 {
        self.0.slot()
    }
}

/// The kind of native view to materialize. Intentionally tiny for now; new kinds
/// are added here and matched in each backend (open/closed: the engine is closed
/// to modification, backends extend by handling new variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    /// A layout container (maps to a plain `UIView` / `ViewGroup`).
    View,
    /// A text label (maps to `UILabel` / `TextView`).
    Text,
}

/// A single settable property on a widget.
///
/// A flat enum (rather than per-widget typed structs) keeps the backend boundary
/// simple and the mutation stream trivially comparable in tests. Type-safe,
/// per-widget builders live one layer up; they lower to these.
#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    /// Text content (valid on [`WidgetKind::Text`]).
    Text(String),
    /// Font size in logical pixels.
    FontSize(f32),
    /// Foreground / text color.
    TextColor(Color),
    /// Background fill.
    BackgroundColor(Color),
    /// Inner padding.
    Padding(EdgeInsets),
}

/// One atomic change to the native view tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Mutation {
    /// Allocate a native view of `kind` for `id`.
    Create {
        /// The new widget's id.
        id: WidgetId,
        /// What to create.
        kind: WidgetKind,
    },
    /// Set or update a property on an existing widget.
    SetAttribute {
        /// Target widget.
        id: WidgetId,
        /// Property to apply.
        attr: Attribute,
    },
    /// Insert `child` into `parent`'s child list at `index`.
    InsertChild {
        /// Container.
        parent: WidgetId,
        /// Position among siblings.
        index: usize,
        /// Child to insert.
        child: WidgetId,
    },
    /// Detach `child` from `parent` (the child may still be re-inserted).
    RemoveChild {
        /// Container.
        parent: WidgetId,
        /// Child to detach.
        child: WidgetId,
    },
    /// Free the native view backing `id`.
    Destroy {
        /// Widget to destroy.
        id: WidgetId,
    },
}
