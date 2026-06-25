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

/// Parameters of a [`spring`] animation: a damped harmonic oscillator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// Restoring force toward the target (higher = snappier).
    pub stiffness: f32,
    /// Resistance that removes energy (higher = less bouncy).
    pub damping: f32,
    /// Mass of the body (higher = slower to accelerate).
    pub mass: f32,
}

impl Default for Spring {
    /// A gentle, slightly-bouncy default (à la react-spring).
    fn default() -> Self {
        Spring {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
}

impl Spring {
    /// A stiff, snappy spring with no overshoot.
    pub const STIFF: Spring = Spring {
        stiffness: 210.0,
        damping: 30.0,
        mass: 1.0,
    };

    /// A loose, wobbly spring.
    pub const WOBBLY: Spring = Spring {
        stiffness: 180.0,
        damping: 12.0,
        mass: 1.0,
    };
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

struct SpringAnim {
    signal: Signal<f32>,
    target: f32,
    position: f32,
    velocity: f32,
    spring: Spring,
}

impl SpringAnim {
    /// Integrates the spring by `dt` seconds (sub-stepped for stability);
    /// returns `true` once it has settled at the target.
    fn advance(&mut self, dt: f32) -> bool {
        // Fixed 240 Hz sub-steps keep semi-implicit Euler stable for stiff
        // springs even at low frame rates.
        let steps = (dt * 240.0).ceil().max(1.0);
        let h = dt / steps;
        for _ in 0..(steps as u32) {
            let force =
                -self.spring.stiffness * (self.position - self.target) - self.spring.damping * self.velocity;
            let accel = force / self.spring.mass.max(0.0001);
            self.velocity += accel * h;
            self.position += self.velocity * h;
        }
        let settled = (self.position - self.target).abs() < 0.01 && self.velocity.abs() < 0.05;
        if settled {
            self.position = self.target;
            self.velocity = 0.0;
        }
        self.signal.set(self.position);
        settled
    }
}

struct Decay {
    signal: Signal<f32>,
    position: f32,
    velocity: f32,
    /// Per-millisecond velocity retention (e.g. `0.998`).
    deceleration: f32,
}

impl Decay {
    /// Integrates velocity decay by `dt` seconds; returns `true` once it stops.
    fn advance(&mut self, dt: f32) -> bool {
        let steps = (dt * 240.0).ceil().max(1.0);
        let h = dt / steps;
        for _ in 0..(steps as u32) {
            self.position += self.velocity * h;
            self.velocity *= self.deceleration.powf(h * 1000.0);
        }
        self.signal.set(self.position);
        self.velocity.abs() < 1.0
    }
}

enum Animation {
    Tween(Tween),
    Spring(SpringAnim),
    Decay(Decay),
}

impl Animation {
    fn advance(&mut self, dt: f32) -> bool {
        match self {
            Animation::Tween(t) => t.advance(dt),
            Animation::Spring(s) => s.advance(dt),
            Animation::Decay(d) => d.advance(dt),
        }
    }
}

thread_local! {
    static ACTIVE: RefCell<Vec<Animation>> = const { RefCell::new(Vec::new()) };
}

/// Starts an animation from `from` to `to` over `duration` seconds with `easing`,
/// returning a signal that carries the animated value.
pub fn animate(from: f32, to: f32, duration: f32, easing: Easing) -> Signal<f32> {
    let signal = rax_reactive::create_signal(from);
    ACTIVE.with(|a| {
        a.borrow_mut().push(Animation::Tween(Tween {
            signal,
            from,
            to,
            duration,
            elapsed: 0.0,
            easing,
        }));
    });
    signal
}

/// Starts a spring animation from `from` to `to` with `spring` physics,
/// returning a signal that carries the animated value. Unlike [`animate`], a
/// spring has no fixed duration — it settles naturally and may overshoot.
///
/// ```
/// use rax_anim::{spring, tick, Spring};
/// use rax_reactive::create_root;
///
/// let (s, scope) = create_root(|| spring(0.0, 100.0, Spring::default()));
/// assert_eq!(s.get(), 0.0);
/// for _ in 0..600 { tick(1.0 / 60.0); } // run to rest
/// assert!((s.get() - 100.0).abs() < 0.1);
/// scope.dispose();
/// ```
pub fn spring(from: f32, to: f32, spring: Spring) -> Signal<f32> {
    let signal = rax_reactive::create_signal(from);
    ACTIVE.with(|a| {
        a.borrow_mut().push(Animation::Spring(SpringAnim {
            signal,
            target: to,
            position: from,
            velocity: 0.0,
            spring,
        }));
    });
    signal
}

/// Starts a decay (fling) animation from `from` with an initial `velocity`
/// (units per second), coasting to a stop. `deceleration` is the per-millisecond
/// velocity retention (`0.998` ≈ a normal scroll fling; smaller stops sooner).
/// Returns a signal carrying the position.
///
/// ```
/// use rax_anim::{decay, tick};
/// use rax_reactive::create_root;
///
/// let (p, scope) = create_root(|| decay(0.0, 1200.0, 0.998));
/// for _ in 0..600 { tick(1.0 / 60.0); }
/// assert!(p.get() > 0.0); // coasted forward then stopped
/// scope.dispose();
/// ```
pub fn decay(from: f32, velocity: f32, deceleration: f32) -> Signal<f32> {
    let signal = rax_reactive::create_signal(from);
    ACTIVE.with(|a| {
        a.borrow_mut().push(Animation::Decay(Decay {
            signal,
            position: from,
            velocity,
            deceleration,
        }));
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

// ── Composition helpers ───────────────────────────────────────────────────────

/// Runs two animated values in parallel.
///
/// Because [`animate`] / [`spring`] / [`decay`] all start immediately upon
/// call and return independent [`Signal`]s, running them in parallel is the
/// default behaviour. This function is a *documentation helper*: it takes two
/// already-started signals and returns them as a tuple, making intent explicit
/// in code.
///
/// # Example
/// ```rust
/// use rax_anim::{animate, parallel, Easing};
/// use rax_reactive::create_root;
///
/// let ((x, y), scope) = create_root(|| {
///     parallel(
///         animate(0.0, 100.0, 0.3, Easing::EaseOut),
///         animate(0.0,  50.0, 0.3, Easing::EaseOut),
///     )
/// });
/// scope.dispose();
/// ```
pub fn parallel<A: 'static, B: 'static>(a: Signal<A>, b: Signal<B>) -> (Signal<A>, Signal<B>) {
    (a, b)
}

/// Runs a second animation after a first one completes.
///
/// Watches `first` until it reaches `to` (within 0.01 units), then calls
/// `second` once. The check runs inside a reactive effect so it fires
/// automatically on every signal update.
///
/// # Limitations
/// This is a best-effort helper: it triggers `second` the first time `first`
/// stabilises near `to`. If the animated value overshoots (e.g. a spring) or
/// never exactly reaches `to`, the threshold (`0.01`) may need tuning. For
/// frame-perfect sequencing, use a timer future via `rax_async::spawn_local`.
///
/// # Example
/// ```rust
/// use rax_anim::{animate, sequence, tick, Easing};
/// use rax_reactive::{create_root, create_signal};
///
/// let (second_started, scope) = create_root(|| {
///     let flag = create_signal(false);
///     let first = animate(0.0, 100.0, 0.5, Easing::Linear);
///     sequence(first, 100.0, move || flag.set(true));
///     flag
/// });
/// for _ in 0..60 { tick(1.0 / 60.0); }
/// assert!(second_started.get());
/// scope.dispose();
/// ```
pub fn sequence(first: Signal<f32>, to: f32, second: impl FnOnce() + 'static) {
    use std::cell::Cell;
    let fired = std::rc::Rc::new(Cell::new(false));
    let second = std::cell::RefCell::new(Some(second));
    rax_reactive::create_effect(move || {
        if fired.get() {
            return;
        }
        if (first.get() - to).abs() < 0.01 {
            fired.set(true);
            if let Some(f) = second.borrow_mut().take() {
                f();
            }
        }
    });
}

/// Staggers `n` animations, calling `make_anim(i)` for each index.
///
/// **Timer support pending.** True staggering (starting animation `i` only
/// after `delay_ms * i` milliseconds) requires wall-clock timer callbacks,
/// which are not yet available in `rax_anim`. This function currently calls
/// `make_anim` for every index **immediately**, so all animations start at the
/// same time. The `delay_ms` parameter is accepted but ignored.
///
/// Once `rax_async` timer primitives are stable, this will be updated to
/// honour the delay without breaking callers.
///
/// # Example
/// ```rust
/// use rax_anim::{stagger, animate, Easing};
/// use rax_reactive::create_root;
///
/// let (signals, scope) = create_root(|| {
///     stagger(3, 50, |i| animate(0.0, 100.0, 0.3, Easing::EaseOut))
/// });
/// scope.dispose();
/// ```
pub fn stagger<F>(n: usize, _delay_ms: u32, mut make_anim: F) -> Vec<Signal<f32>>
where
    F: FnMut(usize) -> Signal<f32>,
{
    (0..n).map(|i| make_anim(i)).collect()
}

/// Creates an oscillating animation that bounces between `from` and `to`.
///
/// The returned [`Signal`] carries the current animated position. Internally
/// the function uses a direction flag and nested reactive effects:
///
/// 1. An outer effect watches `dir` (true = forward, false = reverse) and
///    starts a new [`animate`] leg each time the direction flips.
/// 2. An inner effect copies the current leg's value into the returned signal
///    and flips `dir` when the leg nears its target, which re-triggers step 1.
///
/// This approach works entirely within the existing reactive/animation
/// scheduler — no timers needed. Accuracy depends on frame rate; the
/// completion threshold is `0.5` units.
///
/// # Example
/// ```rust
/// use rax_anim::{oscillate, tick, Easing};
/// use rax_reactive::create_root;
///
/// let (v, scope) = create_root(|| oscillate(0.0, 100.0, 1.0, Easing::Linear));
/// tick(0.6);          // approaching 60
/// let mid = v.get();
/// assert!(mid > 0.0 && mid < 100.0, "mid={mid}");
/// scope.dispose();
/// ```
pub fn oscillate(from: f32, to: f32, duration: f32, easing: Easing) -> Signal<f32> {
    let v = rax_reactive::create_signal(from);
    let dir = rax_reactive::create_signal(true); // true = forward

    rax_reactive::create_effect(move || {
        let forward = dir.get();
        let (a, b) = if forward { (from, to) } else { (to, from) };
        let anim_val = animate(a, b, duration, easing);
        // Inner effect: mirror value and flip direction when the leg ends.
        rax_reactive::create_effect(move || {
            let cur = anim_val.get();
            v.set(cur);
            let target = if forward { to } else { from };
            if (cur - target).abs() < 0.5 {
                dir.update(|d| *d = !*d);
            }
        });
    });
    v
}

/// Returns a signal that starts animating only after `start_trigger` becomes
/// `true`.
///
/// Until the trigger fires the signal holds `from`. Once `start_trigger` is
/// `true`, [`animate`] is started and the returned signal mirrors its value.
///
/// # Limitations
/// For precise wall-clock delays, combine a timer future from
/// `rax_async::spawn_local` with a simple `Signal<bool>` trigger instead.
///
/// # Example
/// ```rust
/// use rax_anim::{delayed, tick, Easing};
/// use rax_reactive::{create_root, create_signal};
///
/// let (val, scope) = create_root(|| {
///     let trigger = create_signal(false);
///     let v = delayed(trigger, 0.0, 100.0, 0.5, Easing::Linear);
///     trigger.set(true);   // fire the trigger
///     v
/// });
/// tick(0.5);
/// assert!((val.get() - 100.0).abs() < 0.5);
/// scope.dispose();
/// ```
pub fn delayed(
    start_trigger: Signal<bool>,
    from: f32,
    to: f32,
    duration: f32,
    easing: Easing,
) -> Signal<f32> {
    let value = rax_reactive::create_signal(from);
    rax_reactive::create_effect(move || {
        if start_trigger.get() {
            let anim = animate(from, to, duration, easing);
            rax_reactive::create_effect(move || value.set(anim.get()));
        }
    });
    value
}

#[cfg(test)]
mod tests;
