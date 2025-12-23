//! Optional resolver module for pattern-based content transformation.
//!
//! This module provides a pattern for implementing content resolution,
//! such as transforming short references to full paths/URLs.
//!
//! # Usage Pattern
//!
//! 1. Define your reference types (prefixes, patterns)
//! 2. Implement parsing logic
//! 3. Implement resolution logic
//! 4. Integrate with preprocessor
//!
//! # Example
//!
//! ```rust,ignore
//! // In preprocessor.rs:
//! use crate::resolver::{find_references, ReferenceResolver};
//!
//! fn process_chapter(chapter: &mut Chapter, book_root: &Path) -> Result<(), Error> {
//!     let resolver = ReferenceResolver::new(book_root);
//!     let references = find_references(&chapter.content);
//!
//!     // Replace from end to start to preserve indices
//!     let mut content = chapter.content.clone();
//!     for (start, end, reference) in references.into_iter().rev() {
//!         if let Some(resolved) = resolver.resolve(&reference) {
//!             content.replace_range(start..end, &resolved);
//!         }
//!     }
//!     chapter.content = content;
//!     Ok(())
//! }
//! ```

use std::path::Path;

/// Represents a parsed reference from content.
///
/// Customize this struct to match your reference patterns.
/// Example patterns:
/// - `prefix:identifier` (e.g., `adr:0042`, `docs:api/auth`)
/// - `prefix:owner:repo:path` (e.g., `gh:owner:repo:issues/123`)
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    /// The reference prefix/type (e.g., "adr", "docs", "gh")
    pub prefix: String,
    /// The identifier or path after the prefix
    pub identifier: String,
}

impl Reference {
    /// Parse a reference string into a Reference.
    ///
    /// Returns None if the string doesn't match the expected pattern.
    pub fn parse(s: &str) -> Option<Self> {
        // Basic pattern: prefix:identifier
        let (prefix, identifier) = s.split_once(':')?;

        // Validate prefix (customize as needed)
        if prefix.is_empty() || identifier.is_empty() {
            return None;
        }

        Some(Self {
            prefix: prefix.to_string(),
            identifier: identifier.to_string(),
        })
    }
}

/// Find all references in content that match the pattern `](prefix:identifier)`.
///
/// Returns a vector of (start, end, Reference) tuples.
/// The indices are relative to the content string and can be used
/// for in-place replacement.
///
/// # Requires
///
/// Add `regex = "1"` to your Cargo.toml dependencies.
pub fn find_references(content: &str) -> Vec<(usize, usize, Reference)> {
    // Note: Requires regex crate
    // use regex::Regex;
    //
    // let re = Regex::new(r"\]\(([a-zA-Z]+:[^)]+)\)").unwrap();
    // re.captures_iter(content)
    //     .filter_map(|cap| {
    //         let ref_str = cap.get(1)?.as_str();
    //         // Skip URLs (they have :// after the scheme)
    //         if ref_str.contains("://") {
    //             return None;
    //         }
    //         let reference = Reference::parse(ref_str)?;
    //         let ref_match = cap.get(1)?;
    //         Some((ref_match.start(), ref_match.end(), reference))
    //     })
    //     .collect()

    // Placeholder implementation - replace with regex-based implementation
    let _ = content;
    Vec::new()
}

/// Resolver for transforming references into concrete paths/URLs.
///
/// Configure with the book's root path and any lookup directories.
pub struct ReferenceResolver<'a> {
    /// Root path of the book for relative path resolution
    pub book_root: &'a Path,
    // Add additional configuration fields as needed:
    // pub lookup_dirs: HashMap<String, PathBuf>,
    // pub url_templates: HashMap<String, String>,
}

impl<'a> ReferenceResolver<'a> {
    /// Create a new resolver with the book root path.
    pub fn new(book_root: &'a Path) -> Self {
        Self { book_root }
    }

    /// Resolve a reference to its target path or URL.
    ///
    /// Returns None if the reference cannot be resolved.
    pub fn resolve(&self, reference: &Reference) -> Option<String> {
        // Customize resolution logic based on prefix type
        match reference.prefix.as_str() {
            // Example: adr:0042 -> ../adr/0042-*.md
            "adr" => self.resolve_adr(&reference.identifier),

            // Example: docs:api/auth -> ../docs/api/auth.md
            "docs" => self.resolve_docs(&reference.identifier),

            // Example: gh:owner:repo:issues/123 -> https://github.com/owner/repo/issues/123
            "gh" => self.resolve_github(&reference.identifier),

            // Unknown prefix - return None
            _ => None,
        }
    }

    /// Resolve ADR reference (customize as needed).
    fn resolve_adr(&self, identifier: &str) -> Option<String> {
        // Example: look up ADR file by number
        // let pattern = format!("{}/adr/{}-*.md", self.book_root.display(), identifier);
        // glob::glob(&pattern).ok()?.next()?.ok()
        let _ = identifier;
        None
    }

    /// Resolve docs reference (customize as needed).
    fn resolve_docs(&self, identifier: &str) -> Option<String> {
        // Example: map to docs path
        // Some(format!("../docs/{}.md", identifier))
        let _ = identifier;
        None
    }

    /// Resolve GitHub reference (customize as needed).
    fn resolve_github(&self, identifier: &str) -> Option<String> {
        // Example: gh:owner:repo:issues/123 -> https://github.com/owner/repo/issues/123
        // let parts: Vec<&str> = identifier.split(':').collect();
        // match parts.as_slice() {
        //     [owner, repo, path] => Some(format!("https://github.com/{}/{}/{}", owner, repo, path)),
        //     _ => None,
        // }
        let _ = identifier;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_reference() {
        let reference = Reference::parse("adr:0042").unwrap();
        assert_eq!(reference.prefix, "adr");
        assert_eq!(reference.identifier, "0042");
    }

    #[test]
    fn test_parse_path_reference() {
        let reference = Reference::parse("docs:api/auth").unwrap();
        assert_eq!(reference.prefix, "docs");
        assert_eq!(reference.identifier, "api/auth");
    }

    #[test]
    fn test_parse_invalid_empty_prefix() {
        assert!(Reference::parse(":identifier").is_none());
    }

    #[test]
    fn test_parse_invalid_empty_identifier() {
        assert!(Reference::parse("prefix:").is_none());
    }

    #[test]
    fn test_parse_no_colon() {
        assert!(Reference::parse("noprefix").is_none());
    }
}
