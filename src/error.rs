//! Error types for the plugin.

use thiserror::Error;

/// Errors that can occur during plugin execution.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Processing error during chapter transformation
    #[error("Processing error in {chapter}: {message}")]
    Processing {
        chapter: String,
        message: String,
    },

    /// MDBook error wrapper
    #[error("MDBook error: {0}")]
    MdBook(String),

    // TODO: Add plugin-specific error variants here
    // Example:
    // #[error("Parse error at line {line}: {message}")]
    // Parse { line: usize, message: String },
}

impl From<mdbook::errors::Error> for Error {
    fn from(err: mdbook::errors::Error) -> Self {
        Error::MdBook(err.to_string())
    }
}
