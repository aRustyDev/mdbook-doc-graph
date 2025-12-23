//! MDBook plugin for DESCRIPTION.
//!
//! # Configuration
//!
//! Add to your `book.toml`:
//!
//! ```toml
//! [preprocessor.PLUGIN_NAME]
//! # Add configuration options here
//! ```
//!
//! # Usage
//!
//! TODO: Add usage examples

pub mod config;
pub mod error;
pub mod preprocessor;

pub use config::Config;
pub use error::Error;
pub use preprocessor::PluginPreprocessor;
