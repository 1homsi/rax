//! Undo/redo history stack on top of signals.
//!
//! [`History<T>`] wraps three signals — current value, past stack, future stack —
//! and exposes a simple push/undo/redo API. Because [`Signal<T>`] is `Copy`, the
//! struct is `Copy` too, making it ergonomic to pass around in `move` closures
//! without cloning.

use crate::{create_signal, Signal};

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

/// A signal-backed value with undo/redo history.
///
/// # Example
///
/// ```rust,ignore
/// let counter = use_history(0i32);
/// counter.push(1);
/// counter.push(2);
/// assert_eq!(counter.get(), 2);
/// counter.undo();
/// assert_eq!(counter.get(), 1);
/// counter.redo();
/// assert_eq!(counter.get(), 2);
/// ```
pub struct History<T: Clone + PartialEq + 'static> {
    current: Signal<T>,
    past: Signal<Vec<T>>,
    future: Signal<Vec<T>>,
}

impl<T: Clone + PartialEq + 'static> History<T> {
    /// Creates a new history with `initial` as the starting value.
    pub fn new(initial: T) -> Self {
        Self {
            current: create_signal(initial),
            past: create_signal(Vec::new()),
            future: create_signal(Vec::new()),
        }
    }

    /// Reads the current value reactively (subscribes the calling effect/memo).
    pub fn get(&self) -> T {
        self.current.get()
    }

    /// Pushes a new value onto the history stack.
    ///
    /// - If `value` equals the current value the call is a no-op.
    /// - Clears the redo stack (any undone future is discarded).
    pub fn push(&self, value: T) {
        let current = self.current;
        let past = self.past;
        let future = self.future;

        if value == current.get() {
            return;
        }
        let old = current.get();
        past.update(|v| v.push(old));
        future.set(Vec::new()); // clear redo stack
        current.set(value);
    }

    /// Reverts to the previous value.
    ///
    /// Returns `true` if an undo was available, `false` if the history was
    /// already at the oldest entry.
    pub fn undo(&self) -> bool {
        let past = self.past;
        let future = self.future;
        let current = self.current;

        let mut done = false;
        past.update(|v| {
            if let Some(prev) = v.pop() {
                let cur = current.get();
                future.update(|f| f.push(cur));
                current.set(prev);
                done = true;
            }
        });
        done
    }

    /// Re-applies the most recently undone value.
    ///
    /// Returns `true` if a redo was available, `false` if there was nothing to
    /// redo.
    pub fn redo(&self) -> bool {
        let past = self.past;
        let future = self.future;
        let current = self.current;

        let mut done = false;
        future.update(|v| {
            if let Some(next) = v.pop() {
                let cur = current.get();
                past.update(|p| p.push(cur));
                current.set(next);
                done = true;
            }
        });
        done
    }

    /// Returns `true` if [`undo`](Self::undo) would have any effect.
    pub fn can_undo(&self) -> bool {
        !self.past.get().is_empty()
    }

    /// Returns `true` if [`redo`](Self::redo) would have any effect.
    pub fn can_redo(&self) -> bool {
        !self.future.get().is_empty()
    }

    /// Returns the underlying current-value [`Signal`] for reactive reads.
    ///
    /// Subscribing to this signal is equivalent to calling [`get`](Self::get)
    /// inside an effect.
    pub fn signal(&self) -> Signal<T> {
        self.current
    }
}

// `Signal<T>` is Copy + 'static, so all three fields are bitwise-copyable.
impl<T: Clone + PartialEq + 'static> Copy for History<T> {}

impl<T: Clone + PartialEq + 'static> Clone for History<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

/// Creates a signal with undo/redo history tracking.
///
/// This is the primary entry-point for using [`History`] inside a component:
///
/// ```rust,ignore
/// let text = use_history(String::new());
/// text.push("hello".to_string());
/// ```
pub fn use_history<T: Clone + PartialEq + 'static>(initial: T) -> History<T> {
    History::new(initial)
}
