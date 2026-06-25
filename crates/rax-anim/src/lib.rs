//! Animation for `rax`: tweened values that live in signals and are advanced by
//! the frame loop.
//!
//! [`animate`] returns a `Signal<f32>` that interpolates from a start to an end
//! value over a duration with an easing curve. Because it's a signal, any view
//! that reads it (e.g. inside a reactive `text` or a bound attribute) updates
//! automatically as the value changes — fine-grained, no tree diff.
//!
//! The runtime calls [`tick`] once per frame with the elapsed time; tests call
//! it directly with a fixed delta for determinism.
//!
//! ```
//! use rax_anim::{animate, tick, Easing};
//! use rax_reactive::create_root;
//!
//! let (a, scope) = create_root(|| animate(0.0, 100.0, 1.0, Easing::Linear));
//! assert_eq!(a.get(), 0.0);
//! tick(0.5); // halfway
//! assert!((a.get() - 50.0).abs() < 0.01);
//! tick(0.5); // done
//! assert_eq!(a.get(), 100.0);
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use std::cell::RefCell;

use rax_reactive::Signal;

/// Easing curves applied to normalized time `t` in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    /// Constant velocity.
    #[default]
    Linear,
    /// Accelerate from rest.
    EaseIn,
    /// Decelerate to rest.
    EaseOut,
    /// Accelerate then decelerate.
    EaseInOut,
}

impl Easing {
    /// Maps normalized time `t` (`0.0..=1.0`) through the curve.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

struct Tween {
    signal: Signal<f32>,
    from: f32,
    to: f32,
    duration: f32,
    elapsed: f32,
    easing: Easing,
}

impl Tween {
    /// Advances by `dt` seconds; returns `true` when finished.
    fn advance(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        let t = if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).min(1.0)
        };
        let value = self.from + (self.to - self.from) * self.easing.apply(t);
        self.signal.set(value);
        t >= 1.0
    }
}

thread_local! {
    static ACTIVE: RefCell<Vec<Tween>> = const { RefCell::new(Vec::new()) };
}

/// Starts an animation from `from` to `to` over `duration` seconds with `easing`,
/// returning a signal that carries the animated value.
pub fn animate(from: f32, to: f32, duration: f32, easing: Easing) -> Signal<f32> {
    let signal = rax_reactive::create_signal(from);
    ACTIVE.with(|a| {
        a.borrow_mut().push(Tween {
            signal,
            from,
            to,
            duration,
            elapsed: 0.0,
            easing,
        });
    });
    signal
}

/// Advances all active animations by `dt` seconds, dropping finished ones.
/// Called once per frame by the runtime. Returns the number still running.
pub fn tick(dt: f32) -> usize {
    // Take the list out so a `signal.set` (which runs effects that could, in
    // principle, start new animations) cannot alias the borrow.
    let mut tweens = ACTIVE.with(|a| std::mem::take(&mut *a.borrow_mut()));
    tweens.retain_mut(|tween| !tween.advance(dt));
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        // Prepend the still-running ones before any started during advance.
        tweens.append(&mut active);
        *active = tweens;
    });
    ACTIVE.with(|a| a.borrow().len())
}

/// Whether any animation is currently running (the driver can idle otherwise).
pub fn is_animating() -> bool {
    ACTIVE.with(|a| !a.borrow().is_empty())
}

#[cfg(test)]
mod tests;
