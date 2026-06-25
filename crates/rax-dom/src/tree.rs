//! The retained element tree.
//!
//! Each node owns: its kind, parent/child links, and the reactive effects that
//! bind its attributes. When a node is removed we dispose its effects (so they
//! stop emitting) and tear down the subtree depth-first. This is also where the
//! reactive-runtime "ownership" gap noted in `rax-reactive` is closed for the UI:
//! effect lifetime is tied to element lifetime.

use rax_core::Arena;
use rax_reactive::{create_effect, Effect};

use crate::backend::Host;
use crate::mutation::{Attribute, Mutation, WidgetId, WidgetKind};

struct ElementNode {
    // Read back once the reconciler/inspector land (they match on node kind).
    #[allow(dead_code)]
    kind: WidgetKind,
    parent: Option<WidgetId>,
    children: Vec<WidgetId>,
    /// Reactive bindings owned by this node, disposed when it is removed.
    effects: Vec<Effect>,
}

/// The retained element tree, paired with the backend it emits mutations to.
pub struct Tree {
    host: Host,
    nodes: Arena<ElementNode>,
    root: Option<WidgetId>,
}

impl Tree {
    /// Creates an empty tree that emits mutations through `host`.
    pub fn new(host: Host) -> Self {
        Tree {
            host,
            nodes: Arena::new(),
            root: None,
        }
    }

    /// The current root widget, if one has been set.
    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    /// Marks `id` as the tree root. (Backends treat the root specially: it is
    /// attached to the platform's content view rather than to a parent widget.)
    pub fn set_root(&mut self, id: WidgetId) {
        self.root = Some(id);
    }

    /// Creates a layout container view.
    pub fn create_view(&mut self) -> WidgetId {
        self.create(WidgetKind::View)
    }

    /// Creates a text widget.
    pub fn create_text(&mut self) -> WidgetId {
        self.create(WidgetKind::Text)
    }

    fn create(&mut self, kind: WidgetKind) -> WidgetId {
        let index = self.nodes.insert(ElementNode {
            kind,
            parent: None,
            children: Vec::new(),
            effects: Vec::new(),
        });
        let id = WidgetId(index);
        self.host.emit(Mutation::Create { id, kind });
        id
    }

    /// Sets a static attribute that never changes.
    pub fn set(&mut self, id: WidgetId, attr: Attribute) {
        if self.nodes.get(id.0).is_some() {
            self.host.emit(Mutation::SetAttribute { id, attr });
        }
    }

    /// Binds an attribute to a reactive computation.
    ///
    /// `f` is run immediately (emitting the initial value) and re-run whenever a
    /// signal it reads changes — emitting **exactly one** `SetAttribute` per
    /// change. This is the core payoff of fine-grained reactivity: no tree diff,
    /// just a targeted update. The binding lives as long as the widget.
    pub fn bind(&mut self, id: WidgetId, mut f: impl FnMut() -> Attribute + 'static) {
        if self.nodes.get(id.0).is_none() {
            return;
        }
        let host = self.host.clone();
        let effect = create_effect(move || {
            let attr = f();
            host.emit(Mutation::SetAttribute { id, attr });
        });
        // Safe: existence checked above, and ids are stable handles.
        self.nodes.get_mut(id.0).unwrap().effects.push(effect);
    }

    /// Appends `child` as the last child of `parent`.
    pub fn append(&mut self, parent: WidgetId, child: WidgetId) {
        let index = match self.nodes.get(parent.0) {
            Some(p) => p.children.len(),
            None => return,
        };
        self.insert_child(parent, index, child);
    }

    /// Inserts `child` into `parent` at `index`.
    pub fn insert_child(&mut self, parent: WidgetId, index: usize, child: WidgetId) {
        // Validate both endpoints before mutating anything.
        if self.nodes.get(parent.0).is_none() || self.nodes.get(child.0).is_none() {
            return;
        }
        if let Some(c) = self.nodes.get_mut(child.0) {
            c.parent = Some(parent);
        }
        let clamped = {
            let p = self.nodes.get_mut(parent.0).unwrap();
            let i = index.min(p.children.len());
            p.children.insert(i, child);
            i
        };
        self.host.emit(Mutation::InsertChild {
            parent,
            index: clamped,
            child,
        });
    }

    /// Removes `id` and its entire subtree, disposing all reactive bindings and
    /// emitting `RemoveChild` (from its parent) followed by `Destroy` for every
    /// node, children-first.
    pub fn remove(&mut self, id: WidgetId) {
        if self.nodes.get(id.0).is_none() {
            return;
        }

        // Detach from parent's child list first (one RemoveChild for the root of
        // the removed subtree; descendants leave with their parent implicitly).
        let parent = self.nodes.get(id.0).and_then(|n| n.parent);
        if let Some(parent) = parent {
            if let Some(p) = self.nodes.get_mut(parent.0) {
                p.children.retain(|c| *c != id);
            }
            self.host.emit(Mutation::RemoveChild { parent, child: id });
        }

        self.destroy_subtree(id);

        if self.root == Some(id) {
            self.root = None;
        }
    }

    /// Depth-first teardown: dispose effects and emit `Destroy`, children first
    /// so a backend can rely on leaves being gone before their container.
    fn destroy_subtree(&mut self, id: WidgetId) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };
        let children = core::mem::take(&mut node.children);
        let effects = core::mem::take(&mut node.effects);

        for effect in effects {
            effect.dispose();
        }
        for child in children {
            self.destroy_subtree(child);
        }

        self.nodes.remove(id.0);
        self.host.emit(Mutation::Destroy { id });
    }

    // --- introspection (for tests / inspector) -----------------------------

    /// Number of live widgets in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the tree has no widgets.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The children of `id`, in order (empty if `id` is unknown).
    pub fn children_of(&self, id: WidgetId) -> &[WidgetId] {
        match self.nodes.get(id.0) {
            Some(n) => &n.children,
            None => &[],
        }
    }
}
