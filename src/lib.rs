//! MDBook Doc-Graph Preprocessor
//!
//! Builds a comprehensive relationship graph of all documents by parsing
//! frontmatter and inline references. Outputs structured data for visualization,
//! querying, and use by other plugins.
//!
//! # Configuration
//!
//! Add to your `book.toml`:
//!
//! ```toml
//! [preprocessor.doc-graph]
//! command = "mdbook-doc-graph"
//!
//! [preprocessor.doc-graph.output]
//! graph_file = "target/docs-graph.json"
//! stats_file = "target/docs-stats.json"
//! mermaid_file = "target/docs-graph.mmd"
//! ```

pub mod config;
pub mod error;
pub mod extractor;
pub mod graph;
pub mod preprocessor;
pub mod scanner;

pub use config::Config;
pub use error::Error;
pub use graph::{DocGraph, Edge, EdgeMetadata, EdgeType, Node, NodeMetadata, NodeType, Statistics};
pub use preprocessor::DocGraphPreprocessor;
