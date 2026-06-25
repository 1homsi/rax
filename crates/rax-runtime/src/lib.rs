//! The `rax` app driver: it owns the element tree, mounts the root view inside a
//! reactive ownership scope, runs layout, and drains platform events each frame.
//!
//! A platform backend creates an [`App`], hands it the viewport size, pushes
//! events through [`App::event_sink`], and calls [`App::tick`] once per frame
//! (driven by `CADisplayLink`/`Choreographer`). The runtime is intentionally
//! backend-agnostic — it talks only to the [`Host`] and the layout engine.

#![forbid(unsafe_code)]

use std::collections::HashMap;

use rax_core::{Rect, Size};
use rax_dom::{EventSink, Host, Tree, WidgetId};
use rax_reactive::{create_root, Scope};
use rax_view::{mount, View};

/// A running application: a mounted view tree plus the per-frame drive loop.
pub struct App {
    tree: Tree,
    root: WidgetId,
    /// Owns all reactivity created while mounting; disposed when the app drops.
    _scope: Scope,
    viewport: Size,
    /// Last frame emitted per widget, so re-layout only emits real changes.
    frames: HashMap<WidgetId, Rect>,
}

impl App {
    /// Mounts `view` against `host`, performs the initial layout for `viewport`,
    /// and returns the running app.
    pub fn new(host: Host, viewport: Size, view: impl View) -> App {
        let mut tree = Tree::new(host);
        let (root, scope) = create_root(|| mount(&mut tree, view));
        let mut app = App {
            tree,
            root,
            _scope: scope,
            viewport,
            frames: HashMap::new(),
        };
        app.relayout();
        app
    }

    /// The root widget of the mounted tree.
    pub fn root(&self) -> WidgetId {
        self.root
    }

    /// A `Send` handle the backend uses to enqueue platform events.
    pub fn event_sink(&self) -> EventSink {
        self.tree.event_sink()
    }

    /// Updates the viewport size (on rotation/resize) and re-lays-out.
    pub fn set_viewport(&mut self, size: Size) {
        if size != self.viewport {
            self.viewport = size;
            self.relayout();
        }
    }

    /// Advances one frame: deliver queued events (which may write signals and
    /// emit paint mutations synchronously), then re-run layout and emit any
    /// changed frames.
    pub fn tick(&mut self) {
        self.tree.drain_events();
        self.relayout();
    }

    /// Recomputes layout and emits only the frames that actually changed.
    fn relayout(&mut self) {
        let computed = rax_layout::compute(&self.tree, self.root, self.viewport);
        for (id, rect) in computed {
            if self.frames.get(&id) != Some(&rect) {
                self.tree.set_frame(id, rect);
                self.frames.insert(id, rect);
            }
        }
    }
}
