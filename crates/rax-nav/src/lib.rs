//! Stack navigation for `rax`.
//!
//! A [`Navigator`] holds a stack of routes in a signal. [`routes`] renders the
//! top route via a dynamic subtree, so pushing/popping reactively swaps the
//! screen. The navigator is provided via context, so any descendant can call
//! [`use_navigator`] to drive navigation without prop-threading.
//!
//! Routes are your own `Clone` type (usually an enum), so navigation is
//! compile-checked.
//!
//! ```
//! use rax_nav::{create_navigator, routes};
//! use rax_view::{boxed, text, View};
//!
//! #[derive(Clone)]
//! enum Screen { Home, Details(u32) }
//!
//! fn app() -> impl View {
//!     let nav = create_navigator(Screen::Home);
//!     routes(nav, move |screen| match screen {
//!         Screen::Home => boxed(text("home")),
//!         Screen::Details(id) => boxed(text(format!("details {id}"))),
//!     })
//! }
//! ```

#![forbid(unsafe_code)]

use rax_reactive::{create_signal, provide_context, use_context, Signal};
use rax_view::{dynamic, BoxedView, View};

/// A navigation stack over routes of type `R`. Cheap `Copy` handle.
pub struct Navigator<R: 'static> {
    stack: Signal<Vec<R>>,
}

impl<R: 'static> Clone for Navigator<R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: 'static> Copy for Navigator<R> {}

/// Creates a navigator with `initial` as the root route and provides it via
/// context so descendants can [`use_navigator`].
pub fn create_navigator<R: Clone + 'static>(initial: R) -> Navigator<R> {
    let nav = Navigator {
        stack: create_signal(vec![initial]),
    };
    provide_context(nav);
    nav
}

/// The navigator of route type `R` in scope, if any.
pub fn use_navigator<R: Clone + 'static>() -> Option<Navigator<R>> {
    use_context::<Navigator<R>>()
}

impl<R: Clone + 'static> Navigator<R> {
    /// Pushes a new route onto the stack.
    pub fn push(&self, route: R) {
        self.stack.update(|s| s.push(route));
    }

    /// Pops the top route (no-op at the root).
    pub fn pop(&self) {
        self.stack.update(|s| {
            if s.len() > 1 {
                s.pop();
            }
        });
    }

    /// Replaces the top route.
    pub fn replace(&self, route: R) {
        self.stack.update(|s| {
            s.pop();
            s.push(route);
        });
    }

    /// Resets the stack to a single route.
    pub fn reset(&self, route: R) {
        self.stack.update(|s| {
            s.clear();
            s.push(route);
        });
    }

    /// Pops back to the root route.
    pub fn pop_to_root(&self) {
        self.stack.update(|s| s.truncate(1));
    }

    /// The current (top) route. Tracked: reading it in a view re-renders on
    /// navigation.
    pub fn top(&self) -> R {
        self.stack
            .with(|s| s.last().expect("navigator stack is never empty").clone())
    }

    /// Number of routes on the stack (tracked).
    pub fn depth(&self) -> usize {
        self.stack.with(|s| s.len())
    }

    /// Whether there is a route to pop back to (tracked).
    pub fn can_pop(&self) -> bool {
        self.depth() > 1
    }
}

/// Renders the navigator's current route. `render` maps a route to a view; when
/// the stack changes, the displayed screen swaps reactively.
pub fn routes<R, F>(nav: Navigator<R>, mut render: F) -> impl View
where
    R: Clone + 'static,
    F: FnMut(R) -> BoxedView + 'static,
{
    dynamic(move || render(nav.top()))
}
