//! SQLite storage facade.
//!
//! Apple and desktop targets use the bundled `rusqlite` implementation.
//! Android and web keep the module available but return clear errors until
//! platform-backed SQLite, IndexedDB, or OPFS persistence lands.

#![forbid(unsafe_code)]

#[cfg(not(any(
    target_os = "android",
    all(target_arch = "wasm32", target_os = "unknown")
)))]
mod native;

#[cfg(not(any(
    target_os = "android",
    all(target_arch = "wasm32", target_os = "unknown")
)))]
pub use native::*;

#[cfg(any(
    target_os = "android",
    all(target_arch = "wasm32", target_os = "unknown")
))]
mod unsupported {
    /// A reactive query that re-runs when `invalidate()` is called.
    #[derive(Clone, Copy)]
    pub struct ReactiveQuery<T: Clone + 'static> {
        result: crate::reactive::Signal<Vec<T>>,
        version: crate::reactive::Signal<u64>,
    }

    impl<T: Clone + 'static> ReactiveQuery<T> {
        /// Invalidate the cache.
        pub fn invalidate(&self) {
            self.version.update(|v| *v += 1);
        }

        /// Access results reactively.
        pub fn get(&self) -> Vec<T> {
            self.result.get()
        }

        /// Re-run the query now and update the signal.
        pub fn refresh<F: Fn() -> Vec<T>>(&self, fetch: F) {
            self.result.update(|r| *r = fetch());
        }
    }

    /// Create a reactive query seeded with `initial_results`.
    pub fn use_reactive_query<T: Clone + 'static>(initial_results: Vec<T>) -> ReactiveQuery<T> {
        use crate::reactive::create_signal;
        ReactiveQuery {
            result: create_signal(initial_results),
            version: create_signal(0u64),
        }
    }

    /// Placeholder database handle for unsupported storage targets.
    pub struct Database;

    impl Database {
        /// Open a database.
        ///
        /// This returns an error until this target grows a platform storage
        /// backend.
        pub fn open(_path: impl AsRef<std::path::Path>) -> Result<Self, String> {
            Err(unsupported())
        }

        /// Execute a SQL statement.
        pub fn execute(&self, _sql: &str) -> Result<usize, String> {
            Err(unsupported())
        }

        /// Execute a SQL statement with positional parameters.
        pub fn execute_with<T>(&self, _sql: &str, _params: &[T]) -> Result<usize, String> {
            Err(unsupported())
        }

        /// Query rows.
        pub fn query<T, F>(&self, _sql: &str, _map_row: F) -> Result<Vec<T>, String>
        where
            F: Fn(()) -> Result<T, String>,
        {
            Err(unsupported())
        }

        /// Query with positional parameters.
        pub fn query_with<T, P, F>(
            &self,
            _sql: &str,
            _params: &[P],
            _map_row: F,
        ) -> Result<Vec<T>, String>
        where
            F: Fn(()) -> Result<T, String>,
        {
            Err(unsupported())
        }

        /// Apply versioned SQL migrations in order.
        pub fn migrate(&self, _migrations: &[(u32, &str)]) -> Result<(), String> {
            Err(unsupported())
        }

        /// Return a web storage path placeholder.
        pub fn documents_path(filename: &str) -> String {
            filename.to_string()
        }
    }

    fn unsupported() -> String {
        "SQLite on this target requires a platform storage backend".to_string()
    }
}

#[cfg(any(
    target_os = "android",
    all(target_arch = "wasm32", target_os = "unknown")
))]
pub use unsupported::*;
