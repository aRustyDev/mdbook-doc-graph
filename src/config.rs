//! Configuration for the plugin.

use serde::Deserialize;
use toml::value::Table;

/// Plugin configuration from `book.toml`.
///
/// Configured in `book.toml` under `[preprocessor.PLUGIN_NAME]`.
///
/// # Example
///
/// ```toml
/// [preprocessor.PLUGIN_NAME]
/// option1 = "value"
/// option2 = true
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    // TODO: Add configuration fields here
    // Example:
    // pub option1: String,
    // pub option2: bool,
}

impl Config {
    /// Parse configuration from mdbook's preprocessor config table.
    pub fn from_table(table: &Table) -> Result<Self, crate::Error> {
        let value = toml::Value::Table(table.clone());
        value
            .try_into()
            .map_err(|e| crate::Error::Config(format!("Invalid configuration: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        // TODO: Add assertions for default values
        let _ = config;
    }

    #[test]
    fn test_config_from_empty_table() {
        let table = Table::new();
        let config = Config::from_table(&table);
        assert!(config.is_ok());
    }
}
