//! Error types for the doc-graph plugin.

use thiserror::Error;

/// Errors that can occur during plugin execution.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Processing error during chapter transformation
    #[error("Processing error in {chapter}: {message}")]
    Processing { chapter: String, message: String },

    /// MDBook error wrapper
    #[error("MDBook error: {0}")]
    MdBook(String),

    /// Frontmatter parse error
    #[error("Frontmatter parse error in {path}: {message}")]
    FrontmatterParse { path: String, message: String },

    /// Invalid reference format
    #[error("Invalid reference '{reference}' in {path}: {reason}")]
    InvalidReference {
        reference: String,
        path: String,
        reason: String,
    },

    /// Broken reference (target not found)
    #[error("Broken reference in {source_path}: target '{target}' not found")]
    BrokenReference { source_path: String, target: String },

    /// Circular supersedes chain
    #[error("Circular supersedes chain detected: {chain}")]
    CircularSupersedes { chain: String },

    /// Ambiguous reference (multiple matches)
    #[error("Ambiguous reference '{reference}' in {path}: matches {matches:?}")]
    AmbiguousReference {
        reference: String,
        path: String,
        matches: Vec<String>,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parse error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl From<mdbook::errors::Error> for Error {
    fn from(err: mdbook::errors::Error) -> Self {
        Error::MdBook(err.to_string())
    }
}
