use thiserror::Error;

/// An error coming for the work of an output format function.
#[derive(Debug, Error)]
pub enum FormatError {
    /// Error originating in an IO operation, for example file system.
    #[error("error during I/O operation")]
    Io(#[from] std::io::Error),

    #[error("x")]
    Param(String),

    #[error("an error occured")]
    OtherError(Box<dyn std::error::Error>),

    #[error("an error occurred: {0}")]
    Other(String),
}
