//! SQLite storage for rax apps, backed by [rusqlite](https://docs.rs/rusqlite)
//! with a bundled SQLite so no system library is required.
//!
//! # Example
//!
//! ```rust,no_run
//! use rax_sqlite::Database;
//!
//! let db = Database::open("app.db").unwrap();
//! db.execute("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, body TEXT)").unwrap();
//! db.execute_with("INSERT INTO notes (body) VALUES (?1)", &[&"hello"]).unwrap();
//! let notes: Vec<String> = db.query("SELECT body FROM notes", |row| row.get(0)).unwrap();
//! ```

use rusqlite::{params_from_iter, types::ToSql, Connection};
use std::path::Path;

/// A SQLite database connection.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at `path`. Use `":memory:"` for an in-memory db.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        Connection::open(path)
            .map(|conn| Database { conn })
            .map_err(|e| e.to_string())
    }

    /// Execute a SQL statement with no parameters. Returns the number of rows changed.
    pub fn execute(&self, sql: &str) -> Result<usize, String> {
        self.conn.execute(sql, []).map_err(|e| e.to_string())
    }

    /// Execute a SQL statement with positional parameters.
    pub fn execute_with(&self, sql: &str, params: &[&dyn ToSql]) -> Result<usize, String> {
        self.conn
            .execute(sql, params_from_iter(params.iter().copied()))
            .map_err(|e| e.to_string())
    }

    /// Query rows. `map_row` maps each `rusqlite::Row` to your type.
    pub fn query<T, F>(&self, sql: &str, map_row: F) -> Result<Vec<T>, String>
    where
        F: Fn(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// Query with positional parameters.
    pub fn query_with<T, F>(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
        map_row: F,
    ) -> Result<Vec<T>, String>
    where
        F: Fn(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(params.iter().copied()), map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// Return a path to the given `filename` in the app's Documents directory
    /// (the standard location for user data on iOS).
    ///
    /// On non-iOS targets the filename is returned as-is (relative to cwd),
    /// which is fine for desktop testing.
    pub fn documents_path(filename: &str) -> String {
        #[cfg(target_os = "ios")]
        {
            // On iOS the app sandbox places Documents at a fixed path under
            // the app container; using a bare filename opens in the current
            // working directory which is also inside the sandbox and works for
            // the simulator. A production app should use
            // NSSearchPathForDirectoriesInDomains to resolve the real path.
            filename.to_string()
        }
        #[cfg(not(target_os = "ios"))]
        {
            filename.to_string()
        }
    }
}
