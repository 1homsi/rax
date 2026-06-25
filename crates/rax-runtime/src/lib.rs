//! The `rax` app driver: it owns the element tree, mounts the root view inside a
//! reactive ownership scope, runs layout, and drains platform events each frame.
//!
//! A platform backend creates an [`App`], hands it the viewport size, pushes
//! events through [`App::event_sink`], and calls [`App::tick`] once per frame
//! (driven by `CADisplayLink`/`Choreographer`). The runtime is intentionally
//! backend-agnostic — it talks only to the [`Host`] and the layout engine.

#![forbid(unsafe_code)]

use std::collections::HashMap;

use rax_core::{EdgeInsets, Rect, Size};
use rax_dom::{EventSink, Host, Tree, WidgetId, WidgetKind};
use rax_reactive::{create_root, Scope};
use rax_view::{mount, View};

/// A running application: a mounted view tree plus the per-frame drive loop.
pub struct App {
    tree: Tree,
    root: WidgetId,
    /// Owns all reactivity created while mounting; disposed when the app drops.
    _scope: Scope,
    viewport: Size,
    /// Safe-area insets (notch, status bar, home indicator) reported by the
    /// platform. The root is laid out within the safe region and offset by the
    /// top-left inset, so apps never hardcode notch/home-indicator padding.
    safe_area: EdgeInsets,
    /// Last frame emitted per widget, so re-layout only emits real changes.
    frames: HashMap<WidgetId, Rect>,
    /// Last content size emitted per scroll widget.
    content_sizes: HashMap<WidgetId, Size>,
    /// Wall-clock of the previous tick, for animation deltas.
    last_tick: Option<std::time::Instant>,
}

impl App {
    /// Mounts the view produced by `make_view` against `host`, performs the
    /// initial layout for `viewport`, and returns the running app.
    ///
    /// `make_view` runs **inside** the app's reactive root scope, so any
    /// `provide_context` / theming / navigator setup it performs is visible to
    /// the whole tree.
    pub fn new<V: View>(host: Host, viewport: Size, make_view: impl FnOnce() -> V) -> App {
        let mut tree = Tree::new(host);
        let (root, scope) = create_root(|| mount(&mut tree, make_view()));
        let mut app = App {
            tree,
            root,
            _scope: scope,
            viewport,
            safe_area: EdgeInsets::ZERO,
            frames: HashMap::new(),
            content_sizes: HashMap::new(),
            last_tick: None,
        };
        app.tree.run_dynamic(); // materialize dynamic subtrees before first layout
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

    /// Updates the platform safe-area insets (notch, status bar, home
    /// indicator) and re-lays-out so content stays clear of them.
    pub fn set_safe_area(&mut self, insets: EdgeInsets) {
        if insets != self.safe_area {
            self.safe_area = insets;
            self.relayout();
        }
    }

    /// Advances one frame: deliver queued events (which may write signals and
    /// emit paint mutations synchronously), then re-run layout and emit any
    /// changed frames.
    pub fn tick(&mut self) {
        rax_async::run_until_stalled(); // advance async tasks (may resolve resources)

        // Advance animations by the wall-clock delta since the last frame.
        let now = std::time::Instant::now();
        let dt = self
            .last_tick
            .map(|prev| now.duration_since(prev).as_secs_f32())
            .unwrap_or(0.0);
        self.last_tick = Some(now);
        rax_anim::tick(dt);

        self.tree.drain_events();
        self.tree.run_dynamic(); // events/async/anim may have dirtied dynamic subtrees
        self.relayout();
    }

    /// Recomputes layout and emits only the frames (and scroll content sizes)
    /// that actually changed.
    fn relayout(&mut self) {
        // Lay the tree out within the safe region, then shift the root by the
        // top-left inset. Children are positioned relative to the root, so they
        // ride along — only the root frame needs the offset.
        let avail = self.viewport.deflate(self.safe_area);
        let computed = rax_layout::compute(&self.tree, self.root, avail);
        for (id, mut layout) in computed {
            if id == self.root {
                layout.frame.origin.x += self.safe_area.left;
                layout.frame.origin.y += self.safe_area.top;
            }
            if self.frames.get(&id) != Some(&layout.frame) {
                self.tree.set_frame(id, layout.frame);
                self.frames.insert(id, layout.frame);
            }
            if self.tree.kind_of(id) == Some(WidgetKind::Scroll)
                && self.content_sizes.get(&id) != Some(&layout.content)
            {
                self.tree.set_content_size(id, layout.content);
                self.content_sizes.insert(id, layout.content);
            }
        }
    }
}
