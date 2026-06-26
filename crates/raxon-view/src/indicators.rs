//! Status indicators: `activity_indicator` (spinner) and `progress` (bar).

use raxon_dom::{Attribute, Tree, WidgetId};

use crate::view::View;

/// A spinning activity indicator. Build via [`activity_indicator`].
pub struct ActivityIndicator;

/// Creates a spinning activity indicator.
pub fn activity_indicator() -> ActivityIndicator {
    ActivityIndicator
}

impl View for ActivityIndicator {
    fn build(self, tree: &mut Tree) -> WidgetId {
        tree.create_activity_indicator()
    }
}

/// A determinate progress bar (`0.0..=1.0`). Build via [`progress`].
pub struct Progress {
    value: f32,
}

/// Creates a progress bar at `value` (`0.0..=1.0`).
pub fn progress(value: f32) -> Progress {
    Progress { value }
}

impl View for Progress {
    fn build(self, tree: &mut Tree) -> WidgetId {
        let id = tree.create_progress();
        tree.set(id, Attribute::FloatValue(self.value));
        id
    }
}
