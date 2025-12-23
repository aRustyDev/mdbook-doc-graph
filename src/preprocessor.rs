//! Preprocessor implementation for doc-graph.

use crate::extractor::Frontmatter;
use crate::graph::{DocGraph, Edge, EdgeType, Node};
use crate::scanner::{find_links, normalize_target, path_to_node_id};
use crate::Config;
use mdbook::book::Book;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Doc-graph preprocessor for MDBook.
pub struct DocGraphPreprocessor;

impl DocGraphPreprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocGraphPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for DocGraphPreprocessor {
    fn name(&self) -> &str {
        "doc-graph"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> mdbook::errors::Result<Book> {
        // Get preprocessor config
        let config = ctx
            .config
            .get_preprocessor(self.name())
            .map(Config::from_table)
            .transpose()
            .map_err(|e| mdbook::errors::Error::msg(e.to_string()))?
            .unwrap_or_default();

        let root = &ctx.root;

        // Build the graph
        let mut graph = DocGraph::new();
        let mut node_by_path: HashMap<String, String> = HashMap::new();
        let mut warnings: Vec<String> = Vec::new();

        // First pass: Extract all nodes from chapters
        Self::extract_nodes(&book, root, &mut graph, &mut node_by_path, &mut warnings);

        // Second pass: Extract edges from frontmatter and inline links
        Self::extract_edges(
            &book,
            root,
            &config,
            &mut graph,
            &node_by_path,
            &mut warnings,
        );

        // Generate inverse edges
        if config.inverse_edges.enabled {
            Self::generate_inverse_edges(&config, &mut graph);
        }

        // Compute statistics
        if config.stats.enabled {
            graph.statistics = Some(graph.compute_statistics());
        }

        // Write output files
        let output_path = root.join(&config.output.graph_file);
        Self::write_graph(&graph, &output_path)?;

        // Write stats file if configured
        if let Some(ref stats_file) = config.output.stats_file {
            if let Some(ref stats) = graph.statistics {
                let stats_path = root.join(stats_file);
                Self::write_stats(stats, &stats_path)?;
            }
        }

        // Write mermaid file if configured
        if let Some(ref mermaid_file) = config.output.mermaid_file {
            let mermaid_path = root.join(mermaid_file);
            Self::write_mermaid(&graph, &mermaid_path)?;
        }

        // Log warnings
        for warning in &warnings {
            log::warn!("{}", warning);
        }

        // Log summary
        log::info!(
            "doc-graph: {} nodes, {} edges written to {}",
            graph.nodes.len(),
            graph.edges.len(),
            config.output.graph_file
        );

        // Return book unchanged (we only generate output files)
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // Support all renderers - we just generate output files
        renderer != "not-supported"
    }
}

impl DocGraphPreprocessor {
    /// Extract nodes from all chapters.
    fn extract_nodes(
        book: &Book,
        _root: &Path,
        graph: &mut DocGraph,
        node_by_path: &mut HashMap<String, String>,
        warnings: &mut Vec<String>,
    ) {
        for item in book.iter() {
            if let BookItem::Chapter(chapter) = item {
                // Skip draft chapters
                if chapter.path.is_none() {
                    continue;
                }

                let path = chapter.path.as_ref().unwrap();
                let path_str = path.to_string_lossy().to_string();

                // Extract frontmatter
                let frontmatter = match Frontmatter::extract(&chapter.content) {
                    Ok(Some(fm)) => fm,
                    Ok(None) => Frontmatter::default(),
                    Err(e) => {
                        warnings.push(format!("{}: {}", path_str, e));
                        Frontmatter::default()
                    }
                };

                // Generate node ID
                let node_id = frontmatter
                    .uuid
                    .clone()
                    .unwrap_or_else(|| path_to_node_id(&path_str));

                // Determine node type
                let node_type = frontmatter.infer_node_type(&path_str);

                // Create node
                let mut node = Node::new(node_id.clone(), path_str.clone(), node_type);

                if let Some(ref uuid) = frontmatter.uuid {
                    node = node.with_uuid(uuid.clone());
                }

                node = node.with_metadata(frontmatter.to_node_metadata());

                // Track path to node ID mapping
                node_by_path.insert(path_str, node_id);

                graph.add_node(node);
            }
        }
    }

    /// Extract edges from frontmatter and inline links.
    fn extract_edges(
        book: &Book,
        _root: &Path,
        _config: &Config,
        graph: &mut DocGraph,
        node_by_path: &HashMap<String, String>,
        warnings: &mut Vec<String>,
    ) {
        for item in book.iter() {
            if let BookItem::Chapter(chapter) = item {
                if chapter.path.is_none() {
                    continue;
                }

                let path = chapter.path.as_ref().unwrap();
                let path_str = path.to_string_lossy().to_string();

                let source_id = match node_by_path.get(&path_str) {
                    Some(id) => id.clone(),
                    None => continue,
                };

                // Extract edges from frontmatter
                if let Ok(Some(fm)) = Frontmatter::extract(&chapter.content) {
                    for related in &fm.related {
                        // Try to resolve target
                        let target_id = Self::resolve_target(&related.target, node_by_path, graph);

                        if let Some(target_id) = target_id {
                            let mut edge =
                                Edge::new(source_id.clone(), target_id, related.edge_type.clone());

                            if let Some(ref note) = related.note {
                                edge.metadata.note = Some(note.clone());
                            }

                            graph.add_edge(edge);
                        } else {
                            warnings.push(format!(
                                "{}: unresolved reference '{}'",
                                path_str, related.target
                            ));
                        }
                    }
                }

                // Extract edges from inline links
                let content_without_fm = Frontmatter::strip_frontmatter(&chapter.content);
                let links = find_links(content_without_fm);

                for link in links {
                    if !link.is_internal {
                        continue;
                    }

                    // Normalize the target path
                    let normalized = normalize_target(&link.target, &path_str);

                    // Try to find the target node
                    let target_id = node_by_path
                        .get(&normalized)
                        .or_else(|| node_by_path.get(&format!("{}.md", normalized)))
                        .cloned();

                    if let Some(target_id) = target_id {
                        // Avoid self-links
                        if target_id == source_id {
                            continue;
                        }

                        // Infer edge type from context
                        let edge_type =
                            crate::scanner::infer_edge_type_from_context(&link.text, &link.target);

                        let edge = Edge::new(source_id.clone(), target_id, edge_type);
                        graph.add_edge(edge);
                    }
                }
            }
        }
    }

    /// Try to resolve a target reference to a node ID.
    fn resolve_target(
        target: &str,
        node_by_path: &HashMap<String, String>,
        graph: &DocGraph,
    ) -> Option<String> {
        // Direct node ID match
        if graph.find_node(target).is_some() {
            return Some(target.to_string());
        }

        // UUID match
        if graph.find_node_by_uuid(target).is_some() {
            return graph.find_node_by_uuid(target).map(|n| n.id.clone());
        }

        // Path match
        if let Some(id) = node_by_path.get(target) {
            return Some(id.clone());
        }

        // Try with .md extension
        if let Some(id) = node_by_path.get(&format!("{}.md", target)) {
            return Some(id.clone());
        }

        // Try partial path matching
        for (path, id) in node_by_path {
            if path.ends_with(target) || path.ends_with(&format!("{}.md", target)) {
                return Some(id.clone());
            }
        }

        None
    }

    /// Generate inverse edges based on configuration.
    fn generate_inverse_edges(config: &Config, graph: &mut DocGraph) {
        let edges_to_add: Vec<Edge> = graph
            .edges
            .iter()
            .filter_map(|edge| {
                let inverse_type = config.get_inverse_edge_type(&edge.edge_type.to_string())?;

                Some(
                    Edge::new(
                        edge.target.clone(),
                        edge.source.clone(),
                        EdgeType::from(inverse_type),
                    )
                    .as_auto_generated(),
                )
            })
            .collect();

        for edge in edges_to_add {
            graph.add_edge(edge);
        }
    }

    /// Write the graph to a JSON file.
    fn write_graph(graph: &DocGraph, path: &Path) -> mdbook::errors::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(graph)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Write statistics to a JSON file.
    fn write_stats(stats: &crate::graph::Statistics, path: &Path) -> mdbook::errors::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(stats)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Write a Mermaid diagram to a file.
    fn write_mermaid(graph: &DocGraph, path: &Path) -> mdbook::errors::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut mermaid = String::from("graph LR\n");

        // Group nodes by type
        let mut by_type: HashMap<String, Vec<&Node>> = HashMap::new();
        for node in &graph.nodes {
            by_type
                .entry(node.node_type.to_string())
                .or_default()
                .push(node);
        }

        // Write subgraphs
        for (node_type, nodes) in &by_type {
            mermaid.push_str(&format!("    subgraph {}\n", node_type.to_uppercase()));
            for node in nodes {
                let label = node.metadata.title.as_deref().unwrap_or(&node.id);
                // Sanitize label for Mermaid
                let label = label.replace('"', "'").replace(['[', ']'], "");
                mermaid.push_str(&format!(
                    "        {}[\"{}\"]\n",
                    node.id.replace('-', "_"),
                    label
                ));
            }
            mermaid.push_str("    end\n");
        }

        // Write edges
        for edge in &graph.edges {
            let source = edge.source.replace('-', "_");
            let target = edge.target.replace('-', "_");
            let edge_label = edge.edge_type.to_string().replace('_', " ");
            mermaid.push_str(&format!("    {} -->|{}| {}\n", source, edge_label, target));
        }

        fs::write(path, mermaid)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook::preprocess::Preprocessor;

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
}
