//! Inline link and reference scanning from markdown content.

use crate::graph::EdgeType;
use regex::Regex;
use std::sync::LazyLock;

/// A discovered link from markdown content.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineLink {
    /// The link text (what the user sees)
    pub text: String,

    /// The link target (URL or path)
    pub target: String,

    /// Whether this is an internal link (relative path)
    pub is_internal: bool,

    /// Position in the content (start, end)
    pub position: (usize, usize),
}

/// Regex patterns for link detection.
static MARKDOWN_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap());

static WIKILINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap());

// Reference-style links are parsed during find_links but the regex is kept for future use
#[allow(dead_code)]
static REFERENCE_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\[([^\]]*)\]").unwrap());

/// Find all links in markdown content.
pub fn find_links(content: &str) -> Vec<InlineLink> {
    let mut links = Vec::new();

    // Standard markdown links: [text](url)
    for cap in MARKDOWN_LINK_RE.captures_iter(content) {
        let text = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let target = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
        let full_match = cap.get(0).unwrap();

        let is_internal = is_internal_link(target);

        links.push(InlineLink {
            text: text.to_string(),
            target: target.to_string(),
            is_internal,
            position: (full_match.start(), full_match.end()),
        });
    }

    // Wiki-style links: [[target]] or [[target|text]]
    for cap in WIKILINK_RE.captures_iter(content) {
        let target = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let text = cap.get(2).map(|m| m.as_str()).unwrap_or(target);
        let full_match = cap.get(0).unwrap();

        links.push(InlineLink {
            text: text.to_string(),
            target: target.to_string(),
            is_internal: true, // Wiki links are always internal
            position: (full_match.start(), full_match.end()),
        });
    }

    links
}

/// Check if a link target is internal (not an external URL).
fn is_internal_link(target: &str) -> bool {
    // External URLs have a scheme
    if target.contains("://") {
        return false;
    }

    // mailto: and tel: are external
    if target.starts_with("mailto:") || target.starts_with("tel:") {
        return false;
    }

    // Anchors only are internal
    if target.starts_with('#') {
        return true;
    }

    // Everything else is internal
    true
}

/// Normalize a link target to a consistent path format.
pub fn normalize_target(target: &str, source_path: &str) -> String {
    // Remove anchor
    let target = target.split('#').next().unwrap_or(target);

    // Handle absolute paths
    if target.starts_with('/') {
        return target.trim_start_matches('/').to_string();
    }

    // Handle relative paths
    if target.starts_with("./") || target.starts_with("../") {
        // Resolve relative to source directory
        let source_dir = std::path::Path::new(source_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));

        let resolved = source_dir.join(target);

        // Normalize the path (remove . and ..)
        let mut components: Vec<&str> = Vec::new();
        for component in resolved.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::Normal(s) => {
                    if let Some(s) = s.to_str() {
                        components.push(s);
                    }
                }
                std::path::Component::CurDir => {}
                _ => {}
            }
        }

        return components.join("/");
    }

    // Simple filename or relative path
    target.to_string()
}

/// Generate a node ID from a file path.
pub fn path_to_node_id(path: &str) -> String {
    // Remove extension
    let without_ext = path
        .strip_suffix(".md")
        .or_else(|| path.strip_suffix(".markdown"))
        .unwrap_or(path);

    // Replace path separators with dashes
    let id = without_ext
        .replace(['/', '\\'], "-")
        .trim_start_matches('-')
        .to_string();

    // Handle ADR patterns: extract number
    if let Some(adr_num) = extract_adr_number(&id) {
        return format!("adr-{}", adr_num);
    }

    id
}

/// Extract ADR number from a path or ID.
fn extract_adr_number(s: &str) -> Option<String> {
    static ADR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"adr[-_]?(\d+)").unwrap());

    ADR_RE.captures(s).map(|cap| {
        cap.get(1)
            .map(|m| format!("{:04}", m.as_str().parse::<u32>().unwrap_or(0)))
            .unwrap_or_default()
    })
}

/// Determine edge type from link context (heuristic).
pub fn infer_edge_type_from_context(text: &str, _target: &str) -> EdgeType {
    let text_lower = text.to_lowercase();

    // Check for explicit relationship keywords
    if text_lower.contains("supersede") || text_lower.contains("replace") {
        return EdgeType::Supersedes;
    }

    if text_lower.contains("implement") {
        return EdgeType::Implements;
    }

    if text_lower.contains("decision") || text_lower.contains("adr") {
        return EdgeType::Decision;
    }

    if text_lower.contains("see also") || text_lower.contains("related") {
        return EdgeType::SeeAlso;
    }

    if text_lower.contains("based on") || text_lower.contains("derived from") {
        return EdgeType::BasedOn;
    }

    // Default to generic link
    EdgeType::LinksTo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_markdown_links() {
        let content = r#"
Check out [the docs](../docs/guide.md) for more info.
Also see [external](https://example.com) resources.
"#;

        let links = find_links(content);
        assert_eq!(links.len(), 2);

        assert_eq!(links[0].text, "the docs");
        assert_eq!(links[0].target, "../docs/guide.md");
        assert!(links[0].is_internal);

        assert_eq!(links[1].text, "external");
        assert_eq!(links[1].target, "https://example.com");
        assert!(!links[1].is_internal);
    }

    #[test]
    fn test_find_wikilinks() {
        let content = r#"
See [[ADR-0042]] for details.
Also check [[some/path/file|the file]].
"#;

        let links = find_links(content);
        assert_eq!(links.len(), 2);

        assert_eq!(links[0].text, "ADR-0042");
        assert_eq!(links[0].target, "ADR-0042");
        assert!(links[0].is_internal);

        assert_eq!(links[1].text, "the file");
        assert_eq!(links[1].target, "some/path/file");
        assert!(links[1].is_internal);
    }

    #[test]
    fn test_is_internal_link() {
        assert!(is_internal_link("./file.md"));
        assert!(is_internal_link("../other/file.md"));
        assert!(is_internal_link("file.md"));
        assert!(is_internal_link("#anchor"));
        assert!(is_internal_link("/absolute/path.md"));

        assert!(!is_internal_link("https://example.com"));
        assert!(!is_internal_link("http://example.com"));
        assert!(!is_internal_link("mailto:test@example.com"));
    }

    #[test]
    fn test_normalize_target() {
        assert_eq!(
            normalize_target("./sibling.md", "docs/current.md"),
            "docs/sibling.md"
        );
        assert_eq!(
            normalize_target("../other/file.md", "docs/sub/current.md"),
            "docs/other/file.md"
        );
        assert_eq!(
            normalize_target("/absolute.md", "any/path.md"),
            "absolute.md"
        );
        assert_eq!(
            normalize_target("file.md#anchor", "dir/current.md"),
            "file.md"
        );
    }

    #[test]
    fn test_path_to_node_id() {
        assert_eq!(path_to_node_id("docs/guide.md"), "docs-guide");
        assert_eq!(path_to_node_id("adr/0042-timeout.md"), "adr-0042");
        assert_eq!(
            path_to_node_id("blog/2024-01-01-post.md"),
            "blog-2024-01-01-post"
        );
    }

    #[test]
    fn test_infer_edge_type() {
        assert_eq!(
            infer_edge_type_from_context("See the decision", "adr-0042"),
            EdgeType::Decision
        );
        assert_eq!(
            infer_edge_type_from_context("This supersedes the old one", "old.md"),
            EdgeType::Supersedes
        );
        assert_eq!(
            infer_edge_type_from_context("See also", "related.md"),
            EdgeType::SeeAlso
        );
        assert_eq!(
            infer_edge_type_from_context("click here", "file.md"),
            EdgeType::LinksTo
        );
    }
}
