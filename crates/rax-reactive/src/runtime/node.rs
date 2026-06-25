//! The reactive graph's node data model.
//!
//! A single node type represents all four roles (signal/memo/effect/scope); the
//! [`NodeKind`] discriminates behaviour. Values are type-erased (`Box<dyn Any>`)
//! so one heterogeneous graph can hold every `T` — the only RTTI in the
//! framework, and never exposed past the typed handles.

use core::any::Any;

use rax_core::Index;

/// Staleness marker for the Clean/Check/Dirty pull algorithm.
///
/// Ordering matters (`Clean < Check < Dirty`): propagation only ever *upgrades*
/// a node, so each is touched at most twice per write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodeState {
    /// Up to date.
    Clean,
    /// A transitive input *might* have changed; verify sources before trusting.
    Check,
    /// A direct input changed; must recompute.
    Dirty,
}

/// What a node is, for scheduling and disposal decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    /// A source cell.
    Signal,
    /// A cached derivation (subscriber + source).
    Memo,
    /// A side-effecting sink.
    Effect,
    /// An ownership scope: owns child nodes, holds no value/computation.
    Scope,
}

/// A memo's recompute closure: given the previous value, return the new boxed
/// value and whether it changed.
pub(crate) type MemoFn = Box<dyn FnMut(Option<Box<dyn Any>>) -> (Box<dyn Any>, bool)>;
/// An effect's side-effecting closure.
pub(crate) type EffectFn = Box<dyn FnMut()>;

/// The computation a node carries (if any).
pub(crate) enum Compute {
    Memo(MemoFn),
    Effect(EffectFn),
    /// Signals and scopes have no computation.
    None,
}

/// One node in the reactive graph.
pub(crate) struct Node {
    pub kind: NodeKind,
    pub state: NodeState,
    pub value: Option<Box<dyn Any>>,
    pub compute: Compute,
    /// Inputs this node reads.
    pub sources: Vec<Index>,
    /// Dependents that read this node.
    pub observers: Vec<Index>,
    /// The scope/computation that created this node (for disposal).
    pub owner: Option<Index>,
    /// Nodes created while this node was the active owner.
    pub owned: Vec<Index>,
    /// Context values provided at this scope (keyed by type), looked up by
    /// walking the owner chain.
    pub contexts: Vec<(core::any::TypeId, Box<dyn Any>)>,
}

impl Node {
    fn bare(
        kind: NodeKind,
        state: NodeState,
        compute: Compute,
        value: Option<Box<dyn Any>>,
    ) -> Self {
        Node {
            kind,
            state,
            value,
            compute,
            sources: Vec::new(),
            observers: Vec::new(),
            owner: None,
            owned: Vec::new(),
            contexts: Vec::new(),
        }
    }

    /// A signal node, initialized and clean.
    pub fn signal(value: Box<dyn Any>) -> Self {
        Node::bare(
            NodeKind::Signal,
            NodeState::Clean,
            Compute::None,
            Some(value),
        )
    }

    /// A memo node — starts `Dirty` so the first read computes it (lazy).
    pub fn memo(f: MemoFn) -> Self {
        Node::bare(NodeKind::Memo, NodeState::Dirty, Compute::Memo(f), None)
    }

    /// An effect node — starts `Dirty`; the caller triggers the first run.
    pub fn effect(f: EffectFn) -> Self {
        Node::bare(NodeKind::Effect, NodeState::Dirty, Compute::Effect(f), None)
    }

    /// An ownership scope node.
    pub fn scope() -> Self {
        Node::bare(NodeKind::Scope, NodeState::Clean, Compute::None, None)
    }
}
