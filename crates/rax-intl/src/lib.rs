//! Internationalization for `rax`.
//!
//! A [`Catalog`] maps message keys to localized strings. It is provided down the
//! tree as a `Signal<Catalog>`, so [`t`] is a reactive read: switching locale is
//! one `signal.set`, and every label that called `t` updates — fine-grained, no
//! tree diff. [`t_args`] does simple `{name}` interpolation.
//!
//! ```
//! use rax_intl::{provide_locale, t, Catalog};
//! use rax_reactive::create_root;
//!
//! let (_, scope) = create_root(|| {
//!     let loc = provide_locale(Catalog::from([("hi", "Hello")]));
//!     assert_eq!(t("hi"), "Hello");
//!     loc.set(Catalog::from([("hi", "Hola")]));
//!     assert_eq!(t("hi"), "Hola");
//! });
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use std::collections::HashMap;

use rax_reactive::{create_signal, provide_context, use_context, Signal};

/// A message catalog: keys → localized strings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    entries: HashMap<String, String>,
}

impl Catalog {
    /// An empty catalog.
    pub fn new() -> Self {
        Catalog::default()
    }

    /// Adds/overrides an entry (builder style).
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.insert(key.into(), value.into());
        self
    }

    /// Looks up `key`, falling back to the key itself if absent.
    pub fn get(&self, key: &str) -> String {
        self.entries
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

impl<K: Into<String>, V: Into<String>, const N: usize> From<[(K, V); N]> for Catalog {
    fn from(pairs: [(K, V); N]) -> Self {
        let entries = pairs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Catalog { entries }
    }
}

/// Provides a catalog to the current scope and descendants, returning the
/// `Signal<Catalog>` so the app can switch locale at runtime.
pub fn provide_locale(catalog: Catalog) -> Signal<Catalog> {
    let signal = create_signal(catalog);
    provide_context(signal);
    signal
}

/// The catalog signal in scope (an empty one is provided if none was).
pub fn use_locale() -> Signal<Catalog> {
    use_context::<Signal<Catalog>>().unwrap_or_else(|| provide_locale(Catalog::default()))
}

/// Translates `key` (a tracked read — callers update when the locale changes).
pub fn t(key: &str) -> String {
    use_locale().with(|c| c.get(key))
}

/// Translates `key`, then substitutes `{name}` placeholders from `args`.
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

#[cfg(test)]
mod tests;
