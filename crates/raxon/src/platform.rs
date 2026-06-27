//! Compile-time platform detection helpers.
//!
//! Exposes simple `const` booleans and a string so you can branch on platform
//! at compile time without cluttering your code with `cfg!` macros.
//!
//! # Example
//! ```rust
//! use raxon::platform::{IS_IOS, IS_MACOS, IS_WEB, PLATFORM};
//!
//! if IS_IOS {
//!     println!("Running on iOS");
//! }
//! println!("Platform: {PLATFORM}");
//! ```

/// `true` when targeting iOS (`target_os = "ios"`).
pub const IS_IOS: bool = cfg!(target_os = "ios");

/// `true` when targeting Android (`target_os = "android"`).
pub const IS_ANDROID: bool = cfg!(target_os = "android");

/// `true` when targeting macOS (`target_os = "macos"`).
pub const IS_MACOS: bool = cfg!(target_os = "macos");

/// `true` when targeting the browser (`wasm32-unknown-unknown`).
pub const IS_WEB: bool = cfg!(all(target_arch = "wasm32", target_os = "unknown"));

/// `true` in debug builds (`cfg!(debug_assertions)`).
pub const IS_DEBUG: bool = cfg!(debug_assertions);

/// The current target platform as a lowercase string.
///
/// One of `"ios"`, `"android"`, `"macos"`, `"web"`, or `"unknown"`.
pub const PLATFORM: &str = if cfg!(target_os = "ios") {
    "ios"
} else if cfg!(target_os = "android") {
    "android"
} else if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(all(target_arch = "wasm32", target_os = "unknown")) {
    "web"
} else {
    "unknown"
};

/// Returns one of two values depending on whether the current target is iOS.
///
/// This is a zero-cost helper: the unused branch is eliminated at compile time.
///
/// # Example
/// ```rust
/// use raxon::platform::platform_value;
///
/// let padding: f32 = platform_value(16.0, 12.0); // 16 on iOS, 12 elsewhere
/// ```
#[inline(always)]
pub fn platform_value<T>(ios: T, android: T) -> T {
    #[cfg(target_os = "ios")]
    {
        let _ = android;
        ios
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = ios;
        android
    }
}

/// Returns the value matching the current major platform.
///
/// This is a zero-cost helper: on concrete targets the unused branches are
/// eliminated at compile time.
#[inline(always)]
pub fn platform_choice<T>(ios: T, android: T, web: T, other: T) -> T {
    #[cfg(target_os = "ios")]
    {
        let _ = (android, web, other);
        ios
    }
    #[cfg(target_os = "android")]
    {
        let _ = (ios, web, other);
        android
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        let _ = (ios, android, other);
        web
    }
    #[cfg(not(any(
        target_os = "ios",
        target_os = "android",
        all(target_arch = "wasm32", target_os = "unknown")
    )))]
    {
        let _ = (ios, android, web);
        other
    }
}

/// A monotonic instant that works on every target, including the browser.
///
/// `std::time::Instant::now()` **panics on `wasm32-unknown-unknown`** — there is
/// no time source on that target — so any code that reads the clock per frame
/// (animation deltas, cache staleness) must go through this instead. On native
/// targets it wraps [`std::time::Instant`]; on the web it reads `Date.now()`.
///
/// # Example
/// ```rust
/// use raxon::platform::Monotonic;
///
/// let start = Monotonic::now();
/// // ... do work ...
/// let elapsed_secs = Monotonic::now().secs_since(start);
/// assert!(elapsed_secs >= 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Monotonic(f64);

impl Monotonic {
    /// The current monotonic time.
    #[inline]
    pub fn now() -> Self {
        Monotonic(now_millis())
    }

    /// Fractional seconds elapsed from `earlier` to `self`.
    ///
    /// Clamped to `0.0` if `earlier` is later than `self`, so a backwards clock
    /// step (e.g. an NTP adjustment on the web's wall clock) never yields a
    /// negative delta.
    #[inline]
    pub fn secs_since(self, earlier: Monotonic) -> f32 {
        ((self.0 - earlier.0).max(0.0) / 1000.0) as f32
    }

    /// Whole seconds elapsed from `earlier` to `self` (clamped at `0`).
    #[inline]
    pub fn whole_secs_since(self, earlier: Monotonic) -> u64 {
        ((self.0 - earlier.0).max(0.0) / 1000.0) as u64
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[inline]
fn now_millis() -> f64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    // A process-wide epoch so values are comparable across `Monotonic`s.
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[inline]
fn now_millis() -> f64 {
    // `Date.now()` is wall-clock and can step backwards; callers clamp deltas.
    js_sys::Date::now()
}
