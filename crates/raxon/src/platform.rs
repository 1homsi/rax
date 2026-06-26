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
