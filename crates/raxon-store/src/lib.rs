//! Key-value persistence for `rax`, with reactive [`persisted`] signals.
//!
//! [`Storage`] is the backend trait a platform implements (UserDefaults /
//! SharedPreferences / a file). A thread-local current backend (in-memory by
//! default) is used by the free functions and by [`persisted`], which returns a
//! signal seeded from storage that writes itself back whenever it changes.
//!
//! ```
//! use raxon_store::{persisted, store_get};
//! use raxon_reactive::create_root;
//!
//! let (_, scope) = create_root(|| {
//!     let name = persisted("user.name", "Guest");
//!     assert_eq!(name.get(), "Guest");
//!     name.set("Sam".to_string());
//!     assert_eq!(store_get("user.name").as_deref(), Some("Sam"));
//! });
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use raxon_reactive::{create_effect, create_signal, Signal};

/// A key-value storage backend. Methods take `&self`; implementations use
/// interior mutability (so a backend can be shared cheaply).
pub trait Storage {
    /// Reads a value.
    fn get(&self, key: &str) -> Option<String>;
    /// Writes a value.
    fn set(&self, key: &str, value: &str);
    /// Removes a value.
    fn remove(&self, key: &str);
}

/// An in-memory storage backend (the default; also ideal for tests).
#[derive(Default, Clone)]
pub struct MemoryStorage {
    map: Rc<RefCell<HashMap<String, String>>>,
}

impl MemoryStorage {
    /// Creates an empty in-memory store.
    pub fn new() -> Self {
        MemoryStorage::default()
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }
    fn set(&self, key: &str, value: &str) {
        self.map
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }
    fn remove(&self, key: &str) {
        self.map.borrow_mut().remove(key);
    }
}

thread_local! {
    static STORAGE: RefCell<Box<dyn Storage>> = RefCell::new(Box::new(MemoryStorage::new()));
}

/// Installs a storage backend for the current thread (e.g. a platform-backed
/// one at app start).
pub fn set_storage(storage: impl Storage + 'static) {
    STORAGE.with(|s| *s.borrow_mut() = Box::new(storage));
}

/// Reads a value from the current storage.
pub fn store_get(key: &str) -> Option<String> {
    STORAGE.with(|s| s.borrow().get(key))
}

/// Writes a value to the current storage.
pub fn store_set(key: &str, value: &str) {
    STORAGE.with(|s| s.borrow().set(key, value));
}

/// Removes a value from the current storage.
pub fn store_remove(key: &str) {
    STORAGE.with(|s| s.borrow().remove(key));
}

/// A `String` signal seeded from storage (or `default` if absent) that writes
/// its value back to storage whenever it changes.
pub fn persisted(key: &str, default: &str) -> Signal<String> {
    let initial = store_get(key).unwrap_or_else(|| default.to_string());
    let signal = create_signal(initial);
    let key = key.to_string();
    create_effect(move || {
        let value = signal.get();
        store_set(&key, &value);
    });
    signal
}

#[cfg(test)]
mod tests;
