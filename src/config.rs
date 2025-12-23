//! Configuration for the doc-graph plugin.

use serde::Deserialize;
use toml::value::Table;

/// Plugin configuration from `book.toml`.
///
/// Configured in `book.toml` under `[preprocessor.doc-graph]`.
///
/// # Example
///
/// ```toml
/// [preprocessor.doc-graph]
/// command = "mdbook-doc-graph"
///
/// [preprocessor.doc-graph.output]
/// graph_file = "target/docs-graph.json"
/// stats_file = "target/docs-stats.json"
/// mermaid_file = "target/docs-graph.mmd"
///
/// [preprocessor.doc-graph.inverse_edges]
/// enabled = true
///
/// [preprocessor.doc-graph.validation]
/// orphan_detection = true
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Output configuration
    pub output: OutputConfig,

    /// Inverse edge generation settings
    pub inverse_edges: InverseEdgeConfig,

    /// Validation rules
    pub validation: ValidationConfig,

    /// Statistics generation
    pub stats: StatsConfig,
}

/// Output file configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Path to output graph JSON file
    pub graph_file: String,

    /// Path to output statistics JSON file
    pub stats_file: Option<String>,

    /// Path to output Mermaid diagram file
    pub mermaid_file: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            graph_file: "target/docs-graph.json".to_string(),
            stats_file: Some("target/docs-stats.json".to_string()),
            mermaid_file: None,
        }
    }
}

/// Inverse edge generation configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct InverseEdgeConfig {
    /// Whether to generate inverse edges
    pub enabled: bool,

    /// Custom inverse edge type mappings
    pub types: Vec<InverseEdgeMapping>,
}

impl Default for InverseEdgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            types: vec![
                InverseEdgeMapping::new("supersedes", "superseded_by"),
                InverseEdgeMapping::new("decision", "decided_by"),
                InverseEdgeMapping::new("implements", "implemented_by"),
                InverseEdgeMapping::new("resulted_in", "resulted_from"),
                InverseEdgeMapping::new("based_on", "contributed_to"),
                InverseEdgeMapping::new("became", "originated_from"),
                InverseEdgeMapping::new("watches", "watched_by"),
            ],
        }
    }
}

/// Mapping from forward edge type to inverse edge type.
#[derive(Debug, Clone, Deserialize)]
pub struct InverseEdgeMapping {
    /// Forward edge type name
    pub forward: String,

    /// Inverse edge type name
    pub inverse: String,
}

impl InverseEdgeMapping {
    /// Create a new inverse edge mapping.
    pub fn new(forward: &str, inverse: &str) -> Self {
        Self {
            forward: forward.to_string(),
            inverse: inverse.to_string(),
        }
    }
}

/// Validation configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    /// Enable orphan detection
    pub orphan_detection: bool,

    /// Document types that are allowed to be orphaned
    pub orphan_allowed_types: Vec<String>,

    /// Require ADRs to link to documentation
    pub require_adr_doc_link: bool,

    /// How to handle circular supersedes chains: "error", "warn", or "ignore"
    pub circular_supersedes: String,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            orphan_detection: true,
            orphan_allowed_types: vec!["scratch".to_string(), "draft".to_string()],
            require_adr_doc_link: false,
            circular_supersedes: "warn".to_string(),
        }
    }
}

/// Statistics generation configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StatsConfig {
    /// Enable statistics generation
    pub enabled: bool,

    /// Include edge type counts
    pub include_edge_counts: bool,

    /// Include orphan list
    pub include_orphan_list: bool,
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_edge_counts: true,
            include_orphan_list: true,
        }
    }
}

impl Config {
    /// Parse configuration from mdbook's preprocessor config table.
    pub fn from_table(table: &Table) -> Result<Self, crate::Error> {
        let value = toml::Value::Table(table.clone());
        value
            .try_into()
            .map_err(|e| crate::Error::Config(format!("Invalid configuration: {}", e)))
    }

    /// Get the inverse edge type for a forward edge type.
    pub fn get_inverse_edge_type(&self, forward: &str) -> Option<&str> {
        if !self.inverse_edges.enabled {
            return None;
        }

        self.inverse_edges
            .types
            .iter()
            .find(|m| m.forward == forward)
            .map(|m| m.inverse.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.inverse_edges.enabled);
        assert!(config.validation.orphan_detection);
        assert!(config.stats.enabled);
    }

    #[test]
    fn test_config_from_empty_table() {
        let table = Table::new();
        let config = Config::from_table(&table);
        assert!(config.is_ok());
    }

    #[test]
    fn test_inverse_edge_lookup() {
        let config = Config::default();
        assert_eq!(
            config.get_inverse_edge_type("supersedes"),
            Some("superseded_by")
        );
        assert_eq!(config.get_inverse_edge_type("decision"), Some("decided_by"));
        assert_eq!(config.get_inverse_edge_type("unknown"), None);
    }
}
