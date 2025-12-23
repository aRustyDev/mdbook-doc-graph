//! Integration tests for mdbook-doc-graph.
//!
//! These tests verify the plugin works correctly with mdbook.

use mdbook::preprocess::Preprocessor;
use mdbook_doc_graph::DocGraphPreprocessor;

#[test]
fn test_preprocessor_name() {
    let preprocessor = DocGraphPreprocessor::new();
    assert_eq!(preprocessor.name(), "doc-graph");
}

#[test]
fn test_supports_html_renderer() {
    let preprocessor = DocGraphPreprocessor::new();
    assert!(preprocessor.supports_renderer("html"));
}

#[test]
fn test_supports_epub_renderer() {
    let preprocessor = DocGraphPreprocessor::new();
    assert!(preprocessor.supports_renderer("epub"));
}

#[test]
fn test_does_not_support_not_supported() {
    let preprocessor = DocGraphPreprocessor::new();
    assert!(!preprocessor.supports_renderer("not-supported"));
}
