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

/// Optional resolver module for pattern-based content transformation.
///
/// This module provides a reference parsing and resolution pattern useful for plugins
/// that transform short references (e.g., `adr:0042`) into full paths or URLs.
///
/// To use: uncomment the re-exports below and add `regex = "1"` to your dependencies.
pub mod resolver;

pub use config::Config;
pub use error::Error;
pub use preprocessor::PluginPreprocessor;

// Optional: Uncomment to re-export resolver types for external use
// pub use resolver::{find_references, Reference, ReferenceResolver};
