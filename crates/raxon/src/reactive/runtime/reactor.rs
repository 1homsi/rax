//! The reactive graph state and the operations that are pure graph surgery
//! (no user code runs inside them, so they may hold the borrow).
//!
//! The multi-step operations that must *release* the borrow to run user
//! computations (read/recompute/flush) live in [`super::engine`].

use crate::core::{Arena, Index};

use super::node::{Compute, Node, NodeKind, NodeState};

/// One isolated reactive graph. A [`Runtime`](super::Runtime) owns exactly one.
pub(crate) struct Reactor {
    pub nodes: Arena<Node>,
    /// The computation currently running, whose reads are tracked.
    pub observer: Option<Index>,
    /// The scope currently owning newly-created nodes.
    pub owner: Option<Index>,
    /// Effects scheduled to run on the next flush.
    pub effect_queue: Vec<Index>,
    /// Nesting depth of `batch`; effects flush only at depth 0.
    pub batch_depth: usize,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            nodes: Arena::new(),
            observer: None,
            owner: None,
            effect_queue: Vec::new(),
            batch_depth: 0,
        }
    }

    /// Inserts a node and attaches it to the current owner (if any) so it is
    /// disposed when that scope/computation is torn down.
    pub fn insert_node(&mut self, node: Node) -> Index {
        let key = self.nodes.insert(node);
        if let Some(owner) = self.owner {
            if let Some(o) = self.nodes.get_mut(owner) {
                o.owned.push(key);
            }
            if let Some(n) = self.nodes.get_mut(key) {
                n.owner = Some(owner);
            }
        }
        key
    }

    // --- edges -------------------------------------------------------------

    /// Records that `observer` reads `source` (idempotent within a run).
    pub fn subscribe(&mut self, source: Index, observer: Index) {
        if let Some(s) = self.nodes.get_mut(source) {
            if !s.observers.contains(&observer) {
                s.observers.push(observer);
            }
        }
        if let Some(o) = self.nodes.get_mut(observer) {
            if !o.sources.contains(&source) {
                o.sources.push(source);
            }
        }
    }

    /// Detaches `key` from its current sources, so a re-run can collect a fresh
    /// dependency set (correctly handling conditional reads).
    pub fn clear_sources(&mut self, key: Index) {
        let sources = match self.nodes.get_mut(key) {
            Some(n) => core::mem::take(&mut n.sources),
            None => return,
        };
        for s in sources {
            if let Some(sn) = self.nodes.get_mut(s) {
                sn.observers.retain(|o| *o != key);
            }
        }
    }

    // --- disposal (ownership tree) -----------------------------------------

    /// Disposes everything `key` owns, without disposing `key` itself. Called
    /// before re-running a computation so its prior nested reactivity is freed.
    pub fn dispose_children(&mut self, key: Index) {
        let owned = match self.nodes.get_mut(key) {
            Some(n) => core::mem::take(&mut n.owned),
            None => return,
        };
        for child in owned {
            self.dispose(child);
        }
    }

    /// Recursively disposes `key` and everything it owns (children first), then
    /// detaches it from sources, observers, and its owner.
    pub fn dispose(&mut self, key: Index) {
        // Children first.
        let owned = match self.nodes.get_mut(key) {
            Some(n) => core::mem::take(&mut n.owned),
            None => return,
        };
        for child in owned {
            self.dispose(child);
        }

        self.clear_sources(key);

        // Detach from our observers so dangling edges never resolve.
        if let Some(n) = self.nodes.get_mut(key) {
            let observers = core::mem::take(&mut n.observers);
            for o in observers {
                if let Some(on) = self.nodes.get_mut(o) {
                    on.sources.retain(|s| *s != key);
                }
            }
        }

        // Detach from our owner's owned list.
        let owner = self.nodes.get(key).and_then(|n| n.owner);
        if let Some(owner) = owner {
            if let Some(on) = self.nodes.get_mut(owner) {
                on.owned.retain(|c| *c != key);
            }
        }

        self.effect_queue.retain(|e| *e != key);
        self.nodes.remove(key);
    }

    // --- propagation marking -----------------------------------------------

    /// Upgrades `key` to `incoming` and propagates `Check` to dependents.
    ///
    /// An effect is scheduled exactly once, on its first upgrade away from
    /// `Clean` this cycle (whether to `Check` or `Dirty`), so transitive effects
    /// are never missed. Pure marking — runs no user code.
    pub fn notify(&mut self, key: Index, incoming: NodeState) {
        let (was_clean, kind, observers) = match self.nodes.get_mut(key) {
            Some(n) if n.state < incoming => {
                let was_clean = n.state == NodeState::Clean;
                n.state = incoming;
                (was_clean, n.kind, n.observers.clone())
            }
            _ => return,
        };
        if was_clean && kind == NodeKind::Effect {
            self.effect_queue.push(key);
        }
        for o in observers {
            self.notify(o, NodeState::Check);
        }
    }

    // --- small helpers used across the borrow boundary ---------------------

    pub fn take_compute(&mut self, key: Index) -> Compute {
        match self.nodes.get_mut(key) {
            Some(n) => core::mem::replace(&mut n.compute, Compute::None),
            None => Compute::None,
        }
    }

    pub fn state_of(&self, key: Index) -> Option<NodeState> {
        self.nodes.get(key).map(|n| n.state)
    }

    // --- context (provided down the owner chain) ---------------------------

    /// Provides `value` (keyed by `type_id`) at the current owner scope,
    /// replacing any existing value of that type at this scope.
    pub fn provide_context(&mut self, type_id: core::any::TypeId, value: Box<dyn core::any::Any>) {
        if let Some(owner) = self.owner {
            if let Some(node) = self.nodes.get_mut(owner) {
                node.contexts.retain(|(t, _)| *t != type_id);
                node.contexts.push((type_id, value));
            }
        }
    }

    /// Looks up a context value by type, walking from the current owner up the
    /// owner chain (nearest scope wins).
    pub fn lookup_context(&self, type_id: core::any::TypeId) -> Option<&Box<dyn core::any::Any>> {
        let mut cursor = self.owner;
        while let Some(id) = cursor {
            let node = self.nodes.get(id)?;
            if let Some((_, value)) = node.contexts.iter().find(|(t, _)| *t == type_id) {
                return Some(value);
            }
            cursor = node.owner;
        }
        None
    }
}
