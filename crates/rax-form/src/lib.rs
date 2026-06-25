//! Form validation helpers for rax.
//!
//! # Example
//! ```no_run
//! use rax_form::{use_form_field, use_form, Validator};
//!
//! static EMAIL_RULES: &[Validator] = &[Validator::Required, Validator::Email];
//! static PASSWORD_RULES: &[Validator] = &[Validator::Required, Validator::MinLength(8)];
//!
//! let email = use_form_field("", EMAIL_RULES);
//! let password = use_form_field("", PASSWORD_RULES);
//! let form = use_form(vec![email, password]);
//!
//! // In your view:
//! // text_input(email.value, |v| email.set(v))
//! // dynamic(move || if let Some(err) = email.error.get() { text(err) } else { text("") })
//! // button("Submit", move || {
//! //     if form.validate() {
//! //         // submit...
//! //     }
//! // })
//! ```

use rax_reactive::{create_signal, Signal};

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

/// A rule applied to a form field's string value.
pub enum Validator {
    /// Field must not be empty.
    Required,
    /// Value must be at least N characters.
    MinLength(usize),
    /// Value must not exceed N characters.
    MaxLength(usize),
    /// Value must look like an email address.
    Email,
    /// Value must contain the given substring.
    Contains(&'static str),
    /// Custom validation — returns `Some(error_message)` on failure.
    Custom(Box<dyn Fn(&str) -> Option<String>>),
}

impl Validator {
    /// Run this validator against `value`. Returns `None` if valid, or
    /// `Some(error_message)` on failure.
    pub fn validate(&self, value: &str) -> Option<String> {
        match self {
            Validator::Required => {
                if value.trim().is_empty() {
                    Some("This field is required".into())
                } else {
                    None
                }
            }
            Validator::MinLength(n) => {
                if value.len() < *n {
                    Some(format!("Must be at least {n} characters"))
                } else {
                    None
                }
            }
            Validator::MaxLength(n) => {
                if value.len() > *n {
                    Some(format!("Must be at most {n} characters"))
                } else {
                    None
                }
            }
            Validator::Email => {
                let has_at = value.contains('@');
                let has_dot_after_at = value
                    .split_once('@')
                    .map(|(_, domain)| domain.contains('.'))
                    .unwrap_or(false);
                if has_at && has_dot_after_at {
                    None
                } else {
                    Some("Enter a valid email address".into())
                }
            }
            Validator::Contains(pat) => {
                if value.contains(pat) {
                    None
                } else {
                    Some(format!("Must contain '{pat}'"))
                }
            }
            Validator::Custom(f) => f(value),
        }
    }
}

// ---------------------------------------------------------------------------
// FormField
// ---------------------------------------------------------------------------

/// A reactive form field with value, error, and dirty tracking.
///
/// Obtain one via [`use_form_field`]. The handle is `Copy` so it can be
/// moved freely into view closures.
#[derive(Clone, Copy)]
pub struct FormField {
    /// Current string value — wire this to your text input's display.
    pub value: Signal<String>,
    /// Current validation error, or `None` when the field is valid.
    pub error: Signal<Option<String>>,
    /// `true` once the user has interacted with the field (i.e. [`set`](FormField::set) called).
    pub dirty: Signal<bool>,
    // Stored as a static reference so FormField stays Copy.
    validators: &'static [Validator],
}

impl FormField {
    fn new(initial: &str, validators: &'static [Validator]) -> Self {
        Self {
            value: create_signal(initial.to_string()),
            error: create_signal(None),
            dirty: create_signal(false),
            validators,
        }
    }

    /// Update the field value, run validation, and mark the field as dirty.
    ///
    /// Call this from the text input's change handler.
    pub fn set(&self, new_value: String) {
        self.value.set(new_value.clone());
        self.dirty.set(true);
        let error = self.run_validators(&new_value);
        self.error.set(error);
    }

    /// Run validation against the current value without marking the field dirty.
    ///
    /// Returns `true` if all validators pass. Typically called on form submit
    /// to surface errors on untouched fields.
    pub fn validate(&self) -> bool {
        let value = self.value.get();
        let error = self.run_validators(&value);
        let is_valid = error.is_none();
        self.error.set(error);
        is_valid
    }

    /// Returns `true` if the field currently has no validation error.
    ///
    /// This is a non-reactive snapshot read; use `error.get()` inside a memo
    /// or effect if you need reactive tracking.
    pub fn is_valid(&self) -> bool {
        self.error.get().is_none()
    }

    fn run_validators(&self, value: &str) -> Option<String> {
        self.validators.iter().find_map(|v| v.validate(value))
    }
}

/// Create a reactive form field with the given initial value and validator rules.
///
/// `validators` must be a reference to a `'static` slice so the handle stays
/// `Copy`. Declare the rules as a `static`:
///
/// ```no_run
/// use rax_form::{use_form_field, Validator};
///
/// static RULES: &[Validator] = &[Validator::Required, Validator::Email];
/// let email = use_form_field("", RULES);
/// ```
pub fn use_form_field(initial: &str, validators: &'static [Validator]) -> FormField {
    FormField::new(initial, validators)
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

/// Aggregate handle over a collection of [`FormField`]s.
///
/// Obtain one via [`use_form`].
pub struct Form {
    fields: Vec<FormField>,
}

impl Form {
    /// Validate all fields and return `true` if every one passes.
    ///
    /// Errors are written to each field's `error` signal so the view can
    /// react and display them, even for fields the user never touched.
    pub fn validate(&self) -> bool {
        // Fold over all fields so we run every validator (not short-circuit),
        // ensuring all error signals are populated before the view re-renders.
        self.fields.iter().fold(true, |acc, f| f.validate() && acc)
    }

    /// Returns `true` if every field is currently valid.
    ///
    /// This is a snapshot check and does not re-run validators. Useful for
    /// enabling / disabling a submit button reactively via a `create_memo`.
    pub fn is_valid(&self) -> bool {
        self.fields.iter().all(|f| f.is_valid())
    }

    /// Reset all fields: clears values, errors, and dirty state.
    pub fn reset(&self) {
        for f in &self.fields {
            f.value.set(String::new());
            f.error.set(None);
            f.dirty.set(false);
        }
    }
}

/// Create a [`Form`] aggregating the given fields.
///
/// ```no_run
/// use rax_form::{use_form_field, use_form, Validator};
///
/// static EMAIL_RULES: &[Validator] = &[Validator::Required, Validator::Email];
/// static PW_RULES: &[Validator] = &[Validator::Required, Validator::MinLength(8)];
///
/// let email = use_form_field("", EMAIL_RULES);
/// let password = use_form_field("", PW_RULES);
/// let form = use_form(vec![email, password]);
///
/// // On submit:
/// if form.validate() {
///     // all fields are valid — send the request
/// }
/// ```
pub fn use_form(fields: Vec<FormField>) -> Form {
    Form { fields }
}
