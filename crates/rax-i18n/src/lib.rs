//! Internationalization (i18n) for rax.
//!
//! # Quick start
//!
//! ```no_run
//! use rax_i18n::{provide_i18n, use_t, system_locale};
//!
//! // At app start, provide the context with the initial locale.
//! let i18n = provide_i18n("en");
//!
//! // Register translations (can be called from anywhere before first render).
//! i18n.add_locale("en", &[
//!     ("welcome",    "Welcome, {name}!"),
//!     ("item_count", "{count} item|{count} items"),
//! ]);
//! i18n.add_locale("fr", &[
//!     ("welcome",    "Bienvenue, {name}!"),
//!     ("item_count", "{count} article|{count} articles"),
//! ]);
//!
//! // In views — capture by clone so closures stay 'static.
//! let t = use_t();
//! // text(move || t("welcome", &[("name", "Alice")]))
//! // text(move || i18n.t_plural("item_count", count.get() as u32, &[]))
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rax_reactive::{create_signal, provide_context, use_context, Signal};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Key → template string mapping for one locale.
///
/// Template variables use `{name}` syntax.  Plural forms are encoded as
/// `"singular form|plural form"` and selected by [`I18n::t_plural`].
type Catalog = HashMap<String, String>;

// ---------------------------------------------------------------------------
// I18n handle
// ---------------------------------------------------------------------------

/// The reactive i18n context.  Obtain it via [`provide_i18n`] (at the app
/// root) and read it in descendants via [`use_i18n`].
///
/// `I18n` is intentionally `Clone` (not `Copy`) because it wraps an
/// `Arc<Mutex<…>>` catalog store.  Clone it freely into `move` closures.
#[derive(Clone)]
pub struct I18n {
    /// Currently active locale code (e.g. `"en"`, `"fr"`, `"ar"`).
    ///
    /// This is a reactive [`Signal`]; setting it via [`I18n::set_locale`]
    /// automatically re-runs any effect/memo that read the locale.
    pub locale: Signal<String>,
    /// All registered locale catalogs, shared across clones via `Arc`.
    catalogs: Arc<Mutex<HashMap<String, Catalog>>>,
}

impl I18n {
    fn new(initial_locale: &str) -> Self {
        Self {
            locale: create_signal(initial_locale.to_string()),
            catalogs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // -----------------------------------------------------------------------
    // Catalog management
    // -----------------------------------------------------------------------

    /// Register (or extend) translations for `locale`.
    ///
    /// May be called multiple times; later registrations for the same key
    /// overwrite earlier ones.
    ///
    /// ```no_run
    /// # use rax_i18n::provide_i18n;
    /// let i18n = provide_i18n("en");
    /// i18n.add_locale("en", &[("hello", "Hello!")]);
    /// i18n.add_locale("fr", &[("hello", "Bonjour!")]);
    /// ```
    pub fn add_locale(&self, locale: &str, entries: &[(&str, &str)]) {
        let mut map = self.catalogs.lock().expect("i18n catalog lock poisoned");
        let catalog = map.entry(locale.to_string()).or_insert_with(HashMap::new);
        for (k, v) in entries {
            catalog.insert(k.to_string(), v.to_string());
        }
    }

    // -----------------------------------------------------------------------
    // Translation
    // -----------------------------------------------------------------------

    /// Translate `key` for the current locale, substituting `{var}` placeholders
    /// with the corresponding `(var, value)` pairs in `args`.
    ///
    /// Falls back to the `"en"` catalog if the current locale has no entry,
    /// and returns `key` unchanged if neither catalog contains it.
    ///
    /// ```no_run
    /// # use rax_i18n::provide_i18n;
    /// let i18n = provide_i18n("en");
    /// i18n.add_locale("en", &[("greeting", "Hello, {name}!")]);
    /// assert_eq!(i18n.t("greeting", &[("name", "Alice")]), "Hello, Alice!");
    /// ```
    pub fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
        let locale = self.locale.get();
        let template = self.lookup(&locale, key);
        interpolate(&template, args)
    }

    /// Translate `key` choosing singular or plural form based on `count`.
    ///
    /// The template is split on `|`: the left side is singular (`count == 1`),
    /// the right side is plural.  `{count}` in the chosen form is replaced with
    /// the numeric value; other `{var}` placeholders from `args` are also
    /// substituted.
    ///
    /// ```no_run
    /// # use rax_i18n::provide_i18n;
    /// let i18n = provide_i18n("en");
    /// i18n.add_locale("en", &[("items", "{count} item|{count} items")]);
    /// assert_eq!(i18n.t_plural("items", 1, &[]), "1 item");
    /// assert_eq!(i18n.t_plural("items", 5, &[]), "5 items");
    /// ```
    pub fn t_plural(&self, key: &str, count: u32, args: &[(&str, &str)]) -> String {
        let locale = self.locale.get();
        let template = self.lookup(&locale, key);
        let form = plural_form(&template, count);
        let count_str = count.to_string();
        // Substitute {count} first, then caller-supplied args.
        let mut result = form.replace("{count}", &count_str);
        for (k, v) in args {
            result = result.replace(&format!("{{{k}}}"), v);
        }
        result
    }

    // -----------------------------------------------------------------------
    // Locale switching
    // -----------------------------------------------------------------------

    /// Switch the active locale and notify all reactive dependents.
    ///
    /// Any effect or memo that called `i18n.locale.get()` (or `i18n.t(…)`)
    /// while tracking will automatically re-run.
    pub fn set_locale(&self, locale: &str) {
        self.locale.set(locale.to_string());
    }

    // -----------------------------------------------------------------------
    // Locale metadata
    // -----------------------------------------------------------------------

    /// Returns `true` when the current locale is typically written right-to-left.
    ///
    /// Recognised RTL language tags: `ar`, `he`, `fa`, `ur`, `yi`, `ji`, `iw`,
    /// `ps`, `sd`, `ug`.
    pub fn is_rtl(&self) -> bool {
        let locale = self.locale.get();
        RTL_LOCALES.iter().any(|l| locale.starts_with(l))
    }

    // -----------------------------------------------------------------------
    // Number / currency formatting
    // -----------------------------------------------------------------------

    /// Format `n` as a decimal string with `decimals` fractional digits.
    ///
    /// This is a lightweight implementation that does not apply locale-specific
    /// grouping separators.  For full ICU number formatting, see the `icu` crate.
    ///
    /// ```no_run
    /// # use rax_i18n::provide_i18n;
    /// let i18n = provide_i18n("en");
    /// assert_eq!(i18n.format_number(1234.5, 2), "1234.50");
    /// ```
    pub fn format_number(&self, n: f64, decimals: usize) -> String {
        format!("{:.prec$}", n, prec = decimals)
    }

    /// Format `amount` as a currency string with the given symbol prefix.
    ///
    /// ```no_run
    /// # use rax_i18n::provide_i18n;
    /// let i18n = provide_i18n("en");
    /// assert_eq!(i18n.format_currency(9.99, "$"), "$9.99");
    /// ```
    pub fn format_currency(&self, amount: f64, symbol: &str) -> String {
        format!("{}{:.2}", symbol, amount)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Look up `key` in the catalog for `locale`, falling back to `"en"`, then
    /// returning `key` itself when nothing is found.
    fn lookup(&self, locale: &str, key: &str) -> String {
        let map = self.catalogs.lock().expect("i18n catalog lock poisoned");
        map.get(locale)
            .and_then(|cat| cat.get(key))
            .or_else(|| map.get("en").and_then(|cat| cat.get(key)))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

// ---------------------------------------------------------------------------
// RTL locale list
// ---------------------------------------------------------------------------

static RTL_LOCALES: &[&str] = &["ar", "he", "fa", "ur", "yi", "ji", "iw", "ps", "sd", "ug"];

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

/// Replace every `{key}` occurrence in `template` with the matching value from
/// `args`.  Unknown placeholders are left verbatim.
fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, val) in args {
        result = result.replace(&format!("{{{key}}}"), val);
    }
    result
}

/// Given a `"singular|plural"` template, return the correct form for `count`.
/// Templates without `|` are returned as-is for any count.
fn plural_form(template: &str, count: u32) -> &str {
    match template.find('|') {
        Some(idx) if count == 1 => &template[..idx],
        Some(idx) => &template[idx + 1..],
        None => template,
    }
}

// ---------------------------------------------------------------------------
// Context API
// ---------------------------------------------------------------------------

/// Provide the i18n context near the app root.  Returns the [`I18n`] handle
/// so the caller can immediately register locales via [`I18n::add_locale`].
///
/// Panics if there is no reactive scope in effect (i.e., if called outside a
/// [`create_root`](rax_reactive::create_root) or effect).
///
/// ```no_run
/// use rax_i18n::provide_i18n;
///
/// let i18n = provide_i18n("en");
/// i18n.add_locale("en", &[("hi", "Hi!")]);
/// ```
pub fn provide_i18n(initial_locale: &str) -> I18n {
    let i18n = I18n::new(initial_locale);
    provide_context(i18n.clone());
    i18n
}

/// Retrieve the [`I18n`] handle provided by an ancestor call to [`provide_i18n`].
///
/// Panics with a clear message if no provider is in scope, guiding the
/// developer to add [`provide_i18n`] near the app root.
pub fn use_i18n() -> I18n {
    use_context::<I18n>()
        .expect("use_i18n: no I18n in scope — call provide_i18n() near the app root")
}

/// Shorthand accessor: returns a closure that delegates to [`I18n::t`].
///
/// Designed for ergonomic use inside reactive `move ||` view closures:
///
/// ```no_run
/// use rax_i18n::use_t;
///
/// let t = use_t();
/// // text(move || t("welcome", &[("name", "Alice")]))
/// ```
///
/// The returned closure is `Clone` so it can be shared across multiple closures.
pub fn use_t() -> impl Fn(&str, &[(&str, &str)]) -> String + Clone {
    let i18n = use_i18n();
    move |key: &str, args: &[(&str, &str)]| i18n.t(key, args)
}

/// Detect the preferred locale from the `LANG` environment variable.
///
/// Returns the language tag portion of `LANG` (e.g. `"en-US"` from
/// `"en_US.UTF-8"`), converting underscores to hyphens.  Falls back to
/// `"en"` if the variable is absent or empty.
///
/// ```no_run
/// use rax_i18n::{provide_i18n, system_locale};
///
/// let i18n = provide_i18n(&system_locale());
/// ```
pub fn system_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|lang| {
            // "en_US.UTF-8" → take "en_US", convert '_' to '-'
            lang.split('.').next().map(|l| l.replace('_', "-"))
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en".to_string())
}
