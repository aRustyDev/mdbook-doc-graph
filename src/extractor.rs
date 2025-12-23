//! Frontmatter extraction from markdown files.

use crate::graph::{EdgeType, NodeMetadata, NodeType};
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;

/// Parsed frontmatter from a markdown document.
#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    /// Document UUID (optional)
    pub uuid: Option<String>,

    /// Document title
    pub title: Option<String>,

    /// Document type
    pub doc_type: Option<String>,

    /// Document status
    pub status: Option<String>,

    /// Creation date
    pub created: Option<String>,

    /// Tags
    pub tags: Vec<String>,

    /// Related documents
    pub related: Vec<RelatedRef>,

    /// Raw extra fields
    pub extra: HashMap<String, serde_json::Value>,
}

/// A reference to a related document from frontmatter.
#[derive(Debug, Clone)]
pub struct RelatedRef {
    /// Target reference (UUID, path, or ID)
    pub target: String,

    /// Relationship type
    pub edge_type: EdgeType,

    /// Optional note about the relationship
    pub note: Option<String>,
}

/// Raw frontmatter structure for deserialization.
#[derive(Debug, Deserialize)]
struct RawFrontmatter {
    #[serde(alias = "id")]
    uuid: Option<String>,

    title: Option<String>,

    #[serde(alias = "type", alias = "doc_type")]
    r#type: Option<String>,

    status: Option<String>,

    #[serde(alias = "date")]
    created: Option<String>,

    #[serde(default)]
    tags: Vec<String>,

    #[serde(default)]
    related: Vec<RawRelatedRef>,

    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
}

/// Raw related reference for deserialization.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawRelatedRef {
    /// Simple string reference (just a target)
    Simple(String),

    /// Full reference with type and note
    Full {
        target: String,
        #[serde(alias = "type", default = "default_edge_type")]
        r#type: String,
        note: Option<String>,
    },
}

fn default_edge_type() -> String {
    "related_to".to_string()
}

impl Frontmatter {
    /// Extract frontmatter from markdown content.
    ///
    /// Supports YAML frontmatter delimited by `---`.
    pub fn extract(content: &str) -> Result<Option<Self>, Error> {
        let trimmed = content.trim_start();

        // Check for YAML frontmatter delimiter
        if !trimmed.starts_with("---") {
            return Ok(None);
        }

        // Find the closing delimiter
        let after_first = &trimmed[3..];
        let end_pos = after_first
            .find("\n---")
            .or_else(|| after_first.find("\r\n---"));

        let yaml_content = match end_pos {
            Some(pos) => &after_first[..pos],
            None => return Ok(None), // No closing delimiter
        };

        // Skip empty frontmatter
        if yaml_content.trim().is_empty() {
            return Ok(None);
        }

        // Parse YAML
        let raw: RawFrontmatter = serde_yaml::from_str(yaml_content).map_err(|e| {
            Error::FrontmatterParse {
                path: String::new(), // Will be filled in by caller
                message: e.to_string(),
            }
        })?;

        // Convert to Frontmatter
        let related: Vec<RelatedRef> = raw
            .related
            .into_iter()
            .map(|r| match r {
                RawRelatedRef::Simple(target) => RelatedRef {
                    target,
                    edge_type: EdgeType::RelatedTo,
                    note: None,
                },
                RawRelatedRef::Full {
                    target,
                    r#type,
                    note,
                } => RelatedRef {
                    target,
                    edge_type: EdgeType::from(r#type.as_str()),
                    note,
                },
            })
            .collect();

        // Convert extra fields to JSON values
        let extra: HashMap<String, serde_json::Value> = raw
            .extra
            .into_iter()
            .filter_map(|(k, v)| {
                // Skip known fields
                if matches!(
                    k.as_str(),
                    "uuid"
                        | "id"
                        | "title"
                        | "type"
                        | "doc_type"
                        | "status"
                        | "created"
                        | "date"
                        | "tags"
                        | "related"
                ) {
                    return None;
                }

                // Convert YAML value to JSON
                let json_str = serde_yaml::to_string(&v).ok()?;
                let json_val: serde_json::Value = serde_yaml::from_str(&json_str).ok()?;
                Some((k, json_val))
            })
            .collect();

        Ok(Some(Frontmatter {
            uuid: raw.uuid,
            title: raw.title,
            doc_type: raw.r#type,
            status: raw.status,
            created: raw.created,
            tags: raw.tags,
            related,
            extra,
        }))
    }

    /// Get the content without frontmatter.
    pub fn strip_frontmatter(content: &str) -> &str {
        let trimmed = content.trim_start();

        if !trimmed.starts_with("---") {
            return content;
        }

        let after_first = &trimmed[3..];
        if let Some(pos) = after_first.find("\n---") {
            let remainder = &after_first[pos + 4..];
            remainder.trim_start_matches(['\n', '\r'])
        } else if let Some(pos) = after_first.find("\r\n---") {
            let remainder = &after_first[pos + 5..];
            remainder.trim_start_matches(['\n', '\r'])
        } else {
            content
        }
    }

    /// Convert to NodeMetadata.
    pub fn to_node_metadata(&self) -> NodeMetadata {
        NodeMetadata {
            title: self.title.clone(),
            status: self.status.clone(),
            created: self.created.clone(),
            tags: self.tags.clone(),
            audience: Vec::new(),
            level: None,
            extra: self.extra.clone(),
        }
    }

    /// Infer node type from frontmatter or path.
    pub fn infer_node_type(&self, path: &str) -> NodeType {
        // First check explicit type in frontmatter
        if let Some(ref doc_type) = self.doc_type {
            return NodeType::from(doc_type.as_str());
        }

        // Infer from path
        let path_lower = path.to_lowercase();
        if path_lower.contains("/adr/") || path_lower.contains("adr-") {
            NodeType::Adr
        } else if path_lower.contains("/blog/") || path_lower.contains("/posts/") {
            NodeType::Blog
        } else if path_lower.contains("/notes/") || path_lower.contains("/scratch/") {
            NodeType::Note
        } else if path_lower.contains("/docs/") || path_lower.contains("/src/") {
            NodeType::Doc
        } else {
            NodeType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_frontmatter() {
        let content = r#"---
title: My Document
status: draft
tags:
  - rust
  - testing
---

# Content here
"#;

        let fm = Frontmatter::extract(content).unwrap().unwrap();
        assert_eq!(fm.title, Some("My Document".to_string()));
        assert_eq!(fm.status, Some("draft".to_string()));
        assert_eq!(fm.tags, vec!["rust", "testing"]);
    }

    #[test]
    fn test_extract_with_uuid() {
        let content = r#"---
id: 550e8400-e29b-41d4-a716-446655440000
title: ADR-0042
type: adr
---
"#;

        let fm = Frontmatter::extract(content).unwrap().unwrap();
        assert_eq!(
            fm.uuid,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
        assert_eq!(fm.doc_type, Some("adr".to_string()));
    }

    #[test]
    fn test_extract_with_related() {
        let content = r#"---
title: Implementation Doc
related:
  - target: adr-0042
    type: decision
    note: Explains the timeout behavior
  - simple-ref
---
"#;

        let fm = Frontmatter::extract(content).unwrap().unwrap();
        assert_eq!(fm.related.len(), 2);
        assert_eq!(fm.related[0].target, "adr-0042");
        assert_eq!(fm.related[0].edge_type, EdgeType::Decision);
        assert_eq!(
            fm.related[0].note,
            Some("Explains the timeout behavior".to_string())
        );
        assert_eq!(fm.related[1].target, "simple-ref");
        assert_eq!(fm.related[1].edge_type, EdgeType::RelatedTo);
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\n\nSome content.";
        let fm = Frontmatter::extract(content).unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n# Content";
        let fm = Frontmatter::extract(content).unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn test_strip_frontmatter() {
        let content = r#"---
title: Test
---

# Heading

Content here."#;

        let stripped = Frontmatter::strip_frontmatter(content);
        assert!(stripped.starts_with("# Heading"));
    }

    #[test]
    fn test_infer_node_type_from_path() {
        let fm = Frontmatter::default();

        assert_eq!(fm.infer_node_type("docs/adr/0042-auth.md"), NodeType::Adr);
        assert_eq!(
            fm.infer_node_type("blog/posts/2024-01-01.md"),
            NodeType::Blog
        );
        assert_eq!(fm.infer_node_type("notes/scratch/idea.md"), NodeType::Note);
        assert_eq!(fm.infer_node_type("docs/src/user/guide.md"), NodeType::Doc);
    }

    #[test]
    fn test_infer_node_type_from_frontmatter() {
        let mut fm = Frontmatter::default();
        fm.doc_type = Some("adr".to_string());

        // Frontmatter type takes precedence over path
        assert_eq!(fm.infer_node_type("random/path.md"), NodeType::Adr);
    }
}
