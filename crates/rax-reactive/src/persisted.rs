//! Persisted signals — automatically save/restore values via a KV store.
//!
//! The persisted value is encoded as a string via `Display`/`FromStr`.
//!
//! # Example
//! ```no_run
//! use rax_reactive::persisted_signal;
//! // Creates a signal backed by persistent storage under key "theme_mode":
//! let theme = persisted_signal("theme_mode", "light");
//! // When changed, the new value is saved immediately.
//! theme.set("dark".to_string());
//! // On next app launch, the value is restored from storage.
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{create_effect, create_signal, Signal};

// In-memory KV store backing persisted signals.
// In production, this should be bridged to the platform's UserDefaults / SharedPreferences.
thread_local! {
    static KV_STORE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Write a value to the KV store under `key`.
pub fn kv_set(key: &str, value: &str) {
    KV_STORE.with(|s| s.borrow_mut().insert(key.to_string(), value.to_string()));
}

/// Read a value from the KV store.
pub fn kv_get(key: &str) -> Option<String> {
    KV_STORE.with(|s| s.borrow().get(key).cloned())
}

/// Create a `Signal<String>` whose value is persisted across app sessions
/// under `key`. If a stored value exists, it is used as the initial value;
/// otherwise `default` is used. Changes automatically update the KV store.
pub fn persisted_signal(key: &'static str, default: &str) -> Signal<String> {
    let initial = kv_get(key).unwrap_or_else(|| default.to_string());
    let sig = create_signal(initial);
    create_effect(move || {
        let value = sig.get();
        kv_set(key, &value);
    });
    sig
}

/// Create a `Signal<bool>` backed by persistent storage.
pub fn persisted_bool(key: &'static str, default: bool) -> Signal<bool> {
    let initial = kv_get(key)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default);
    let sig = create_signal(initial);
    create_effect(move || {
        kv_set(key, &sig.get().to_string());
    });
    sig
}

/// Create a `Signal<i64>` backed by persistent storage.
pub fn persisted_i64(key: &'static str, default: i64) -> Signal<i64> {
    let initial = kv_get(key)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default);
    let sig = create_signal(initial);
    create_effect(move || {
        kv_set(key, &sig.get().to_string());
    });
    sig
}

/// Create a `Signal<f64>` backed by persistent storage.
pub fn persisted_f64(key: &'static str, default: f64) -> Signal<f64> {
    let initial = kv_get(key)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default);
    let sig = create_signal(initial);
    create_effect(move || {
        kv_set(key, &sig.get().to_string());
    });
    sig
}
