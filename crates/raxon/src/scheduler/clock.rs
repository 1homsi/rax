//! Time abstraction. The scheduler never reads the wall clock directly so that
//! frame timing is deterministic in tests.

use std::cell::Cell;

use crate::platform::Monotonic;

/// A monotonic time source, in nanoseconds.
pub trait Clock {
    /// Monotonic nanoseconds since some fixed, arbitrary epoch. Must be
    /// non-decreasing across calls on the same clock.
    fn now_nanos(&self) -> u64;
}

/// A clock you advance by hand — the default for tests and headless rendering.
#[derive(Debug, Default)]
pub struct ManualClock {
    nanos: Cell<u64>,
}

impl ManualClock {
    /// A clock starting at `t = 0`.
    pub fn new() -> Self {
        ManualClock {
            nanos: Cell::new(0),
        }
    }

    /// Advances the clock by `delta` nanoseconds.
    pub fn advance(&self, delta: u64) {
        self.nanos.set(self.nanos.get().saturating_add(delta));
    }

    /// Advances the clock by `millis` milliseconds (convenience).
    pub fn advance_millis(&self, millis: u64) {
        self.advance(millis.saturating_mul(1_000_000));
    }
}

impl Clock for ManualClock {
    fn now_nanos(&self) -> u64 {
        self.nanos.get()
    }
}

/// A real monotonic clock backed by [`Monotonic`], for production drivers.
///
/// Works on every target, including the browser — unlike a raw
/// [`std::time::Instant`], which panics on `wasm32-unknown-unknown`.
#[derive(Debug)]
pub struct SystemClock {
    start: Monotonic,
}

impl Default for SystemClock {
    fn default() -> Self {
        SystemClock::new()
    }
}

impl SystemClock {
    /// Creates a clock whose epoch is now.
    pub fn new() -> Self {
        SystemClock {
            start: Monotonic::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now_nanos(&self) -> u64 {
        (Monotonic::now().secs_since(self.start) as f64 * 1_000_000_000.0) as u64
    }
}
