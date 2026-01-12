use thiserror::Error;

/// Application-specific error type
#[derive(Debug, Error)]
pub enum GrindstoneError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not determine data directory")]
    NoDataDirectory,
}

/// Convenience type alias for Result with GrindstoneError
pub type Result<T> = std::result::Result<T, GrindstoneError>;
