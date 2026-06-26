//! Filesystem helpers for rax apps.
//!
//! Provides standard app directory paths and simple read/write operations.
//! All paths are platform-appropriate.
//!
//! # Example
//! ```no_run
//! let docs = app_documents_dir();
//! let path = docs.join("notes.txt");
//! write_text(&path, "Hello")?;
//! let text = read_text(&path)?;
//! ```

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// App directory paths
// ---------------------------------------------------------------------------

/// Returns the app's documents directory (user-visible, backed up on iOS).
///
/// On iOS: `$HOME/Documents`
/// On other platforms: the current executable's parent directory.
pub fn app_documents_dir() -> PathBuf {
    #[cfg(target_os = "ios")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join("Documents");
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the app's cache directory (not backed up, can be purged by OS).
///
/// On iOS: `$HOME/Library/Caches`
pub fn app_cache_dir() -> PathBuf {
    #[cfg(target_os = "ios")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join("Library").join("Caches");
        }
    }
    std::env::temp_dir()
}

/// Returns the app's temporary directory.
pub fn app_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Returns the app's support directory (internal app data, backed up on iOS).
///
/// On iOS: `$HOME/Library/Application Support`
pub fn app_support_dir() -> PathBuf {
    #[cfg(target_os = "ios")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join("Library").join("Application Support");
        }
    }
    app_documents_dir()
}

// ---------------------------------------------------------------------------
// File operations
// ---------------------------------------------------------------------------

/// Read the entire contents of a file as a UTF-8 string.
pub fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Read the entire contents of a file as bytes.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| e.to_string())
}

/// Write a UTF-8 string to a file (creates or overwrites).
pub fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Write bytes to a file (creates or overwrites).
pub fn write_bytes(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, data).map_err(|e| e.to_string())
}

/// Append text to a file (creates if it doesn't exist).
pub fn append_text(path: &Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())
}

/// Delete a file. Returns Ok(()) if the file didn't exist.
pub fn delete_file(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// List all files in a directory (non-recursive). Returns file names only.
pub fn list_files(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if let Ok(name) = entry.file_name().into_string() {
            names.push(name);
        }
    }
    Ok(names)
}

/// Check if a file or directory exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Return the size of a file in bytes.
pub fn file_size(path: &Path) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

/// Create a directory and all its parents.
pub fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}
