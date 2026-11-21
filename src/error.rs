//! Error types for the folio component.

/// Errors in file browsing operations.
#[derive(Debug, thiserror::Error)]
pub enum FolioError {
    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Path is not a directory.
    #[error("not a directory: {0}")]
    NotDirectory(String),

    /// Navigation target invalid.
    #[error("navigation error: {0}")]
    Navigation(String),
}
