//! Graph data structures for document relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete document graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocGraph {
    /// Schema version
    pub version: String,

    /// When the graph was generated
    pub generated_at: DateTime<Utc>,

    /// All document nodes
    pub nodes: Vec<Node>,

    /// All relationship edges
    pub edges: Vec<Edge>,

    /// Graph statistics (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,
}

impl DocGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            nodes: Vec::new(),
            edges: Vec::new(),
            statistics: None,
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Find a node by its ID.
    pub fn find_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find a node by its path.
    pub fn find_node_by_path(&self, path: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.path == path)
    }

    /// Find a node by its UUID.
    pub fn find_node_by_uuid(&self, uuid: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.uuid.as_deref() == Some(uuid))
    }

    /// Get all edges originating from a node.
    pub fn edges_from(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    /// Get all edges pointing to a node.
    pub fn edges_to(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    /// Find orphaned nodes (no incoming or outgoing edges).
    pub fn find_orphans(&self) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|node| {
                !self
                    .edges
                    .iter()
                    .any(|e| e.source == node.id || e.target == node.id)
            })
            .collect()
    }

    /// Compute statistics for the graph.
    pub fn compute_statistics(&self) -> Statistics {
        let mut nodes_by_type: HashMap<String, usize> = HashMap::new();
        let mut edges_by_type: HashMap<String, usize> = HashMap::new();

        for node in &self.nodes {
            *nodes_by_type.entry(node.node_type.to_string()).or_insert(0) += 1;
        }

        for edge in &self.edges {
            *edges_by_type.entry(edge.edge_type.to_string()).or_insert(0) += 1;
        }

        let orphaned_nodes: Vec<String> =
            self.find_orphans().iter().map(|n| n.path.clone()).collect();

        Statistics {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            nodes_by_type,
            edges_by_type,
            orphaned_nodes,
        }
    }
}

impl Default for DocGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A document node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier (derived from path or explicit)
    pub id: String,

    /// Optional UUID from frontmatter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    /// File path relative to book root
    pub path: String,

    /// Document type
    #[serde(rename = "type")]
    pub node_type: NodeType,

    /// Document metadata
    pub metadata: NodeMetadata,
}

impl Node {
    /// Create a new node with the given ID and path.
    pub fn new(id: String, path: String, node_type: NodeType) -> Self {
        Self {
            id,
            uuid: None,
            path,
            node_type,
            metadata: NodeMetadata::default(),
        }
    }

    /// Set the UUID.
    pub fn with_uuid(mut self, uuid: String) -> Self {
        self.uuid = Some(uuid);
        self
    }

    /// Set the metadata.
    pub fn with_metadata(mut self, metadata: NodeMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Document type classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// Architecture Decision Record
    Adr,
    /// Documentation
    Doc,
    /// Blog post
    Blog,
    /// Note or scratch file
    Note,
    /// Unknown type
    Unknown,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Adr => write!(f, "adr"),
            NodeType::Doc => write!(f, "doc"),
            NodeType::Blog => write!(f, "blog"),
            NodeType::Note => write!(f, "note"),
            NodeType::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for NodeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "adr" => NodeType::Adr,
            "doc" | "docs" | "documentation" => NodeType::Doc,
            "blog" | "post" => NodeType::Blog,
            "note" | "notes" | "scratch" => NodeType::Note,
            _ => NodeType::Unknown,
        }
    }
}

/// Metadata associated with a node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Document title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Document status (e.g., "draft", "accepted", "superseded")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Creation date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Tags associated with the document
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Target audience
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audience: Vec<String>,

    /// Difficulty level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,

    /// Additional custom metadata
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A relationship edge between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID
    pub source: String,

    /// Target node ID
    pub target: String,

    /// Edge type
    #[serde(rename = "type")]
    pub edge_type: EdgeType,

    /// Edge metadata
    pub metadata: EdgeMetadata,
}

impl Edge {
    /// Create a new edge.
    pub fn new(source: String, target: String, edge_type: EdgeType) -> Self {
        Self {
            source,
            target,
            edge_type,
            metadata: EdgeMetadata::default(),
        }
    }

    /// Set the metadata.
    pub fn with_metadata(mut self, metadata: EdgeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Mark this edge as auto-generated.
    pub fn as_auto_generated(mut self) -> Self {
        self.metadata.auto = true;
        self.metadata.generated_at = Some(Utc::now());
        self
    }
}

/// Edge type classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    // Hierarchical
    Supersedes,
    SupersededBy,
    ParentOf,
    ChildOf,

    // Semantic
    Decision,
    DecidedBy,
    Implements,
    ImplementedBy,
    SeeAlso,
    RelatedTo,

    // Temporal
    ResultedIn,
    ResultedFrom,
    BasedOn,
    ContributedTo,
    Became,
    OriginatedFrom,
    CoversPeriod,

    // Code watching
    Watches,
    WatchedBy,

    // Generic link (from inline markdown links)
    LinksTo,

    // Custom type
    Custom(String),
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Supersedes => write!(f, "supersedes"),
            EdgeType::SupersededBy => write!(f, "superseded_by"),
            EdgeType::ParentOf => write!(f, "parent_of"),
            EdgeType::ChildOf => write!(f, "child_of"),
            EdgeType::Decision => write!(f, "decision"),
            EdgeType::DecidedBy => write!(f, "decided_by"),
            EdgeType::Implements => write!(f, "implements"),
            EdgeType::ImplementedBy => write!(f, "implemented_by"),
            EdgeType::SeeAlso => write!(f, "see_also"),
            EdgeType::RelatedTo => write!(f, "related_to"),
            EdgeType::ResultedIn => write!(f, "resulted_in"),
            EdgeType::ResultedFrom => write!(f, "resulted_from"),
            EdgeType::BasedOn => write!(f, "based_on"),
            EdgeType::ContributedTo => write!(f, "contributed_to"),
            EdgeType::Became => write!(f, "became"),
            EdgeType::OriginatedFrom => write!(f, "originated_from"),
            EdgeType::CoversPeriod => write!(f, "covers_period"),
            EdgeType::Watches => write!(f, "watches"),
            EdgeType::WatchedBy => write!(f, "watched_by"),
            EdgeType::LinksTo => write!(f, "links_to"),
            EdgeType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for EdgeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "supersedes" => EdgeType::Supersedes,
            "superseded_by" => EdgeType::SupersededBy,
            "parent_of" => EdgeType::ParentOf,
            "child_of" => EdgeType::ChildOf,
            "decision" => EdgeType::Decision,
            "decided_by" => EdgeType::DecidedBy,
            "implements" => EdgeType::Implements,
            "implemented_by" => EdgeType::ImplementedBy,
            "see_also" => EdgeType::SeeAlso,
            "related_to" | "related" => EdgeType::RelatedTo,
            "resulted_in" => EdgeType::ResultedIn,
            "resulted_from" => EdgeType::ResultedFrom,
            "based_on" => EdgeType::BasedOn,
            "contributed_to" => EdgeType::ContributedTo,
            "became" => EdgeType::Became,
            "originated_from" => EdgeType::OriginatedFrom,
            "covers_period" => EdgeType::CoversPeriod,
            "watches" => EdgeType::Watches,
            "watched_by" => EdgeType::WatchedBy,
            "links_to" | "link" => EdgeType::LinksTo,
            other => EdgeType::Custom(other.to_string()),
        }
    }
}

/// Metadata associated with an edge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Optional note describing the relationship
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Whether this edge was auto-generated (inverse edge)
    #[serde(default)]
    pub auto: bool,

    /// When the edge was generated (for auto-generated edges)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<DateTime<Utc>>,
}

/// Graph statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Total number of nodes
    pub total_nodes: usize,

    /// Total number of edges
    pub total_edges: usize,

    /// Node counts by type
    pub nodes_by_type: HashMap<String, usize>,

    /// Edge counts by type
    pub edges_by_type: HashMap<String, usize>,

    /// List of orphaned node paths
    pub orphaned_nodes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = DocGraph::new();
        assert_eq!(graph.version, "1.0");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = DocGraph::new();
        let node = Node::new(
            "adr-0042".to_string(),
            "docs/adr/0042.md".to_string(),
            NodeType::Adr,
        );
        graph.add_node(node);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_find_node() {
        let mut graph = DocGraph::new();
        let node = Node::new(
            "adr-0042".to_string(),
            "docs/adr/0042.md".to_string(),
            NodeType::Adr,
        );
        graph.add_node(node);

        assert!(graph.find_node("adr-0042").is_some());
        assert!(graph.find_node("nonexistent").is_none());
    }

    #[test]
    fn test_find_orphans() {
        let mut graph = DocGraph::new();

        // Add two nodes
        graph.add_node(Node::new(
            "node1".to_string(),
            "path1.md".to_string(),
            NodeType::Doc,
        ));
        graph.add_node(Node::new(
            "node2".to_string(),
            "path2.md".to_string(),
            NodeType::Doc,
        ));
        graph.add_node(Node::new(
            "orphan".to_string(),
            "orphan.md".to_string(),
            NodeType::Note,
        ));

        // Connect node1 and node2
        graph.add_edge(Edge::new(
            "node1".to_string(),
            "node2".to_string(),
            EdgeType::LinksTo,
        ));

        let orphans = graph.find_orphans();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].id, "orphan");
    }

    #[test]
    fn test_node_type_from_str() {
        assert_eq!(NodeType::from("adr"), NodeType::Adr);
        assert_eq!(NodeType::from("doc"), NodeType::Doc);
        assert_eq!(NodeType::from("blog"), NodeType::Blog);
        assert_eq!(NodeType::from("note"), NodeType::Note);
        assert_eq!(NodeType::from("unknown_type"), NodeType::Unknown);
    }

    #[test]
    fn test_edge_type_from_str() {
        assert_eq!(EdgeType::from("supersedes"), EdgeType::Supersedes);
        assert_eq!(EdgeType::from("decision"), EdgeType::Decision);
        assert_eq!(EdgeType::from("see_also"), EdgeType::SeeAlso);
        assert_eq!(
            EdgeType::from("custom_type"),
            EdgeType::Custom("custom_type".to_string())
        );
    }

    #[test]
    fn test_compute_statistics() {
        let mut graph = DocGraph::new();

        graph.add_node(Node::new(
            "n1".to_string(),
            "p1.md".to_string(),
            NodeType::Adr,
        ));
        graph.add_node(Node::new(
            "n2".to_string(),
            "p2.md".to_string(),
            NodeType::Adr,
        ));
        graph.add_node(Node::new(
            "n3".to_string(),
            "p3.md".to_string(),
            NodeType::Doc,
        ));
        graph.add_node(Node::new(
            "n4".to_string(),
            "p4.md".to_string(),
            NodeType::Note,
        ));

        graph.add_edge(Edge::new(
            "n1".to_string(),
            "n2".to_string(),
            EdgeType::Supersedes,
        ));
        graph.add_edge(Edge::new(
            "n3".to_string(),
            "n1".to_string(),
            EdgeType::Decision,
        ));

        let stats = graph.compute_statistics();

        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.total_edges, 2);
        assert_eq!(stats.nodes_by_type.get("adr"), Some(&2));
        assert_eq!(stats.nodes_by_type.get("doc"), Some(&1));
        assert_eq!(stats.orphaned_nodes.len(), 1); // n4 is orphaned
    }
}
