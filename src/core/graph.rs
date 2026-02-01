use super::edge::Edge;
use super::node::{Node, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub version: String,
    pub generated_at: String,
    pub generator: String,
    pub language: String,
    pub root_path: String,
    pub stats: GraphStats,
    #[serde(default)]
    pub file_metadata: HashMap<String, FileMetadata>,
    pub git_commit_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub files_parsed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub last_modified: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub metadata: GraphMetadata,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,

    // Indexes for fast querying (not serialized)
    #[serde(skip, default)]
    pub node_by_id: HashMap<String, usize>,
    #[serde(skip, default)]
    pub outgoing: HashMap<String, Vec<usize>>,
    #[serde(skip, default)]
    pub incoming: HashMap<String, Vec<usize>>,
    #[serde(skip, default)]
    pub by_name: HashMap<String, Vec<usize>>,
    #[serde(skip, default)]
    pub by_type: HashMap<NodeType, Vec<usize>>,

    // Track if indices need rebuilding (Phase 1 optimization)
    #[serde(skip, default)]
    pub(crate) indices_dirty: bool,
}

impl CodeGraph {
    pub fn new(root_path: String, language: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            metadata: GraphMetadata {
                version: "1.0.0".to_string(),
                generated_at: now,
                generator: "code-navigator".to_string(),
                language,
                root_path,
                stats: GraphStats {
                    total_nodes: 0,
                    total_edges: 0,
                    files_parsed: 0,
                },
                file_metadata: HashMap::new(),
                git_commit_hash: None,
            },
            nodes: Vec::new(),
            edges: Vec::new(),
            node_by_id: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            by_name: HashMap::new(),
            by_type: HashMap::new(),
            indices_dirty: false,
        }
    }

    /// Create a new graph with pre-allocated capacity (Phase 1 optimization)
    pub fn new_with_capacity(
        root_path: String,
        language: String,
        estimated_nodes: usize,
        estimated_edges: usize,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            metadata: GraphMetadata {
                version: "1.0.0".to_string(),
                generated_at: now,
                generator: "code-navigator".to_string(),
                language,
                root_path,
                stats: GraphStats {
                    total_nodes: 0,
                    total_edges: 0,
                    files_parsed: 0,
                },
                file_metadata: HashMap::new(),
                git_commit_hash: None,
            },
            nodes: Vec::with_capacity(estimated_nodes),
            edges: Vec::with_capacity(estimated_edges),
            node_by_id: HashMap::with_capacity(estimated_nodes),
            outgoing: HashMap::with_capacity(estimated_edges / 2),
            incoming: HashMap::with_capacity(estimated_edges / 2),
            by_name: HashMap::with_capacity(estimated_nodes / 2),
            by_type: HashMap::with_capacity(10),
            indices_dirty: false,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        let idx = self.nodes.len();
        let id = node.id.clone();
        let name = node.name.clone();
        let node_type = node.node_type.clone();

        self.nodes.push(node);
        self.node_by_id.insert(id, idx);
        self.by_name.entry(name).or_default().push(idx);
        self.by_type.entry(node_type).or_default().push(idx);

        self.metadata.stats.total_nodes = self.nodes.len();
    }

    pub fn add_edge(&mut self, edge: Edge) {
        let idx = self.edges.len();
        let from = edge.from.clone();
        let to = edge.to.clone();

        self.edges.push(edge);
        self.outgoing.entry(from).or_default().push(idx);
        self.incoming.entry(to).or_default().push(idx);

        self.metadata.stats.total_edges = self.edges.len();
    }

    /// Ensure indices are up-to-date (Phase 1 optimization: lazy rebuilding)
    pub fn ensure_indices(&mut self) {
        if self.indices_dirty {
            self.build_indexes();
            self.indices_dirty = false;
        }
    }

    pub fn build_indexes(&mut self) {
        self.node_by_id.clear();
        self.by_name.clear();
        self.by_type.clear();
        self.outgoing.clear();
        self.incoming.clear();

        // Build node indexes
        for (idx, node) in self.nodes.iter().enumerate() {
            self.node_by_id.insert(node.id.clone(), idx);
            self.by_name.entry(node.name.clone()).or_default().push(idx);
            self.by_type
                .entry(node.node_type.clone())
                .or_default()
                .push(idx);
        }

        // Build edge indexes
        for (idx, edge) in self.edges.iter().enumerate() {
            self.outgoing
                .entry(edge.from.clone())
                .or_default()
                .push(idx);
            self.incoming.entry(edge.to.clone()).or_default().push(idx);
        }

        self.indices_dirty = false;
    }

    /// Merge another graph into this one (for parallel parsing)
    /// Phase 1 optimization: Incremental index updates instead of full rebuild
    pub fn merge(&mut self, other: CodeGraph) {
        let base_node_idx = self.nodes.len();
        let base_edge_idx = self.edges.len();

        // Extend nodes with incremental index updates
        for (i, node) in other.nodes.into_iter().enumerate() {
            let idx = base_node_idx + i;
            self.node_by_id.insert(node.id.clone(), idx);
            self.by_name.entry(node.name.clone()).or_default().push(idx);
            self.by_type
                .entry(node.node_type.clone())
                .or_default()
                .push(idx);
            self.nodes.push(node);
        }

        // Extend edges with incremental index updates
        for (i, edge) in other.edges.into_iter().enumerate() {
            let idx = base_edge_idx + i;
            self.outgoing
                .entry(edge.from.clone())
                .or_default()
                .push(idx);
            self.incoming.entry(edge.to.clone()).or_default().push(idx);
            self.edges.push(edge);
        }

        self.metadata
            .file_metadata
            .extend(other.metadata.file_metadata);
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.node_by_id.get(id).and_then(|&idx| self.nodes.get(idx))
    }

    pub fn get_nodes_by_name(&self, name: &str) -> Vec<&Node> {
        self.by_name
            .get(name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.nodes.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_nodes_by_type(&self, node_type: &NodeType) -> Vec<&Node> {
        self.by_type
            .get(node_type)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.nodes.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.outgoing
            .get(node_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.edges.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.incoming
            .get(node_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.edges.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Trace dependencies: find all nodes reachable from a given node up to a certain depth
    pub fn trace_dependencies(&self, from_id: &str, max_depth: usize) -> Vec<TraceResult> {
        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.trace_recursive(from_id, 0, max_depth, &mut visited, &mut results);
        results
    }

    fn trace_recursive(
        &self,
        node_id: &str,
        depth: usize,
        max_depth: usize,
        visited: &mut std::collections::HashSet<String>,
        results: &mut Vec<TraceResult>,
    ) {
        if depth >= max_depth || visited.contains(node_id) {
            return;
        }

        visited.insert(node_id.to_string());

        for edge in self.get_outgoing_edges(node_id) {
            results.push(TraceResult {
                from_id: edge.from.clone(),
                to_name: edge.to.clone(),
                edge_type: edge.edge_type.clone(),
                call_site: edge.call_site.clone(),
                file_path: edge.file_path.clone(),
                line: edge.line,
                depth,
            });

            // Try to find the target node and recurse
            if let Some(target_nodes) = self.by_name.get(&edge.to) {
                for &target_idx in target_nodes {
                    if let Some(target_node) = self.nodes.get(target_idx) {
                        self.trace_recursive(
                            &target_node.id,
                            depth + 1,
                            max_depth,
                            visited,
                            results,
                        );
                    }
                }
            }
        }
    }

    /// Find all callers of a function (reverse lookup by name)
    pub fn find_callers(&self, function_name: &str) -> Vec<&Edge> {
        self.incoming
            .get(function_name)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.edges.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find all paths from one node to another
    pub fn find_paths(&self, from_id: &str, to_name: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![from_id.to_string()];
        let mut visited = std::collections::HashSet::new();

        self.find_paths_recursive(
            from_id,
            to_name,
            &mut current_path,
            &mut visited,
            &mut paths,
            max_depth,
            0,
        );

        paths
    }

    #[allow(clippy::too_many_arguments)]
    fn find_paths_recursive(
        &self,
        current_id: &str,
        target_name: &str,
        current_path: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        paths: &mut Vec<Vec<String>>,
        max_depth: usize,
        depth: usize,
    ) {
        if depth >= max_depth {
            return;
        }

        visited.insert(current_id.to_string());

        for edge in self.get_outgoing_edges(current_id) {
            if edge.to == target_name {
                // Found a path!
                let mut complete_path = current_path.clone();
                complete_path.push(edge.to.clone());
                paths.push(complete_path);
                continue;
            }

            // Try to continue the path
            if let Some(target_indices) = self.by_name.get(&edge.to) {
                for &idx in target_indices {
                    if let Some(next_node) = self.nodes.get(idx) {
                        if !visited.contains(&next_node.id) {
                            current_path.push(edge.to.clone());
                            self.find_paths_recursive(
                                &next_node.id,
                                target_name,
                                current_path,
                                visited,
                                paths,
                                max_depth,
                                depth + 1,
                            );
                            current_path.pop();
                        }
                    }
                }
            }
        }

        visited.remove(current_id);
    }

    /// Calculate complexity metrics for a node
    pub fn get_complexity(&self, node_id: &str) -> ComplexityMetrics {
        let fan_out = self.get_outgoing_edges(node_id).len();
        let fan_in = self
            .find_callers(
                self.get_node_by_id(node_id)
                    .map(|n| n.name.as_str())
                    .unwrap_or(""),
            )
            .len();

        ComplexityMetrics {
            fan_in,
            fan_out,
            cyclomatic: fan_out + 1, // Simplified
        }
    }

    /// Find hotspots (most called functions)
    pub fn find_hotspots(&self, limit: usize) -> Vec<HotspotResult> {
        let mut hotspots: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for edge in &self.edges {
            *hotspots.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let mut results: Vec<_> = hotspots
            .into_iter()
            .map(|(name, count)| HotspotResult {
                name,
                call_count: count,
            })
            .collect();

        results.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        results.truncate(limit);
        results
    }

    /// Extract a subgraph rooted at a specific node with given depth
    pub fn extract_subgraph(&self, from_name: &str, max_depth: usize) -> CodeGraph {
        let mut extracted_nodes = Vec::new();
        let mut extracted_edges = Vec::new();
        let mut visited = HashSet::new();
        let mut node_ids_to_include = HashSet::new();

        // Find starting nodes by name
        if let Some(start_nodes) = self.by_name.get(from_name) {
            for &node_idx in start_nodes {
                if let Some(start_node) = self.nodes.get(node_idx) {
                    self.extract_recursive(
                        &start_node.id,
                        0,
                        max_depth,
                        &mut visited,
                        &mut node_ids_to_include,
                    );
                }
            }
        }

        // Collect nodes that should be included
        for node in &self.nodes {
            if node_ids_to_include.contains(&node.id) {
                extracted_nodes.push(node.clone());
            }
        }

        // Collect edges where from is in the subgraph
        for edge in &self.edges {
            if node_ids_to_include.contains(&edge.from) {
                extracted_edges.push(edge.clone());
            }
        }

        let mut subgraph = CodeGraph {
            metadata: crate::core::GraphMetadata {
                version: "1.0.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: "code-navigator-extract".to_string(),
                language: self.metadata.language.clone(),
                root_path: self.metadata.root_path.clone(),
                stats: crate::core::GraphStats {
                    total_nodes: extracted_nodes.len(),
                    total_edges: extracted_edges.len(),
                    files_parsed: 0,
                },
                file_metadata: HashMap::new(),
                git_commit_hash: None,
            },
            nodes: extracted_nodes,
            edges: extracted_edges,
            node_by_id: Default::default(),
            outgoing: Default::default(),
            incoming: Default::default(),
            by_name: Default::default(),
            by_type: Default::default(),
            indices_dirty: true,
        };

        subgraph.build_indexes();
        subgraph
    }

    /// Filter graph based on criteria, returning a new filtered graph
    pub fn filter(
        &self,
        package_filter: Option<&str>,
        type_filter: Option<&NodeType>,
        exclude_tests: bool,
    ) -> CodeGraph {
        let mut filtered_nodes = Vec::new();
        let mut filtered_node_ids = HashSet::new();

        for node in &self.nodes {
            let mut include = true;

            // Apply package filter
            if let Some(package) = package_filter {
                if node.package != package {
                    include = false;
                }
            }

            // Apply type filter
            if let Some(node_type) = type_filter {
                if &node.node_type != node_type {
                    include = false;
                }
            }

            // Exclude tests
            if exclude_tests {
                let file_path_str = node.file_path.to_string_lossy();
                if file_path_str.contains("_test") || file_path_str.contains(".test.") {
                    include = false;
                }
            }

            if include {
                filtered_nodes.push(node.clone());
                filtered_node_ids.insert(node.id.clone());
            }
        }

        // Filter edges to only include edges where both from and to are in filtered nodes
        let filtered_edges: Vec<_> = self
            .edges
            .iter()
            .filter(|e| filtered_node_ids.contains(&e.from))
            .cloned()
            .collect();

        let mut filtered_graph = CodeGraph {
            metadata: crate::core::GraphMetadata {
                version: self.metadata.version.clone(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: "code-navigator-filter".to_string(),
                language: self.metadata.language.clone(),
                root_path: self.metadata.root_path.clone(),
                stats: crate::core::GraphStats {
                    total_nodes: filtered_nodes.len(),
                    total_edges: filtered_edges.len(),
                    files_parsed: 0,
                },
                file_metadata: HashMap::new(),
                git_commit_hash: None,
            },
            nodes: filtered_nodes,
            edges: filtered_edges,
            node_by_id: Default::default(),
            outgoing: Default::default(),
            incoming: Default::default(),
            by_name: Default::default(),
            by_type: Default::default(),
            indices_dirty: true,
        };

        filtered_graph.build_indexes();
        filtered_graph
    }

    fn extract_recursive(
        &self,
        node_id: &str,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<String>,
        node_ids_to_include: &mut HashSet<String>,
    ) {
        if depth > max_depth || visited.contains(node_id) {
            return;
        }

        visited.insert(node_id.to_string());
        node_ids_to_include.insert(node_id.to_string());

        // Traverse outgoing edges
        for edge in self.get_outgoing_edges(node_id) {
            // Try to find target nodes by name
            if let Some(target_nodes) = self.by_name.get(&edge.to) {
                for &target_idx in target_nodes {
                    if let Some(target_node) = self.nodes.get(target_idx) {
                        self.extract_recursive(
                            &target_node.id,
                            depth + 1,
                            max_depth,
                            visited,
                            node_ids_to_include,
                        );
                    }
                }
            }
        }
    }

    /// Remove all nodes and edges from a specific file
    pub fn remove_nodes_from_file(&mut self, file_path: &str) {
        let file_path_normalized = file_path.to_string();

        // Find nodes to remove
        let nodes_to_remove: Vec<String> = self
            .nodes
            .iter()
            .filter(|n| n.file_path.to_string_lossy() == file_path_normalized)
            .map(|n| n.id.clone())
            .collect();

        // Remove nodes
        self.nodes.retain(|n| !nodes_to_remove.contains(&n.id));

        // Remove edges where from node is being removed
        self.edges.retain(|e| !nodes_to_remove.contains(&e.from));

        // Rebuild indexes after removal
        self.build_indexes();
    }

    /// Track which nodes came from which file (for incremental updates)
    pub fn track_file_metadata(&mut self, file_path: &PathBuf, last_modified: String) {
        let file_path_str = file_path.to_string_lossy().to_string();

        // Find all nodes from this file
        let node_ids: Vec<String> = self
            .nodes
            .iter()
            .filter(|n| n.file_path == *file_path)
            .map(|n| n.id.clone())
            .collect();

        self.metadata.file_metadata.insert(
            file_path_str.clone(),
            FileMetadata {
                path: file_path_str,
                last_modified,
                node_ids,
            },
        );
    }

    /// Compare this graph with another and return differences
    pub fn diff(&self, other: &CodeGraph) -> GraphDiff {
        let mut added_nodes = Vec::new();
        let mut removed_nodes = Vec::new();
        let mut changed_nodes = Vec::new();
        let mut complexity_changes = Vec::new();

        // Build sets of node IDs for quick lookup
        let old_ids: HashSet<_> = self.nodes.iter().map(|n| n.id.clone()).collect();
        let new_ids: HashSet<_> = other.nodes.iter().map(|n| n.id.clone()).collect();

        // Find added nodes (in new but not in old)
        for node_id in &new_ids {
            if !old_ids.contains(node_id) {
                added_nodes.push(node_id.clone());
            }
        }

        // Find removed nodes (in old but not in new)
        for node_id in &old_ids {
            if !new_ids.contains(node_id) {
                removed_nodes.push(node_id.clone());
            }
        }

        // Find changed nodes (present in both but with different signatures)
        for old_node in &self.nodes {
            if let Some(new_node) = other.get_node_by_id(&old_node.id) {
                if old_node.signature != new_node.signature || old_node.line != new_node.line {
                    changed_nodes.push(NodeChange {
                        node_id: old_node.id.clone(),
                        node_name: old_node.name.clone(),
                        old_signature: old_node.signature.clone(),
                        new_signature: new_node.signature.clone(),
                        old_line: old_node.line,
                        new_line: new_node.line,
                    });
                }

                // Check complexity changes
                let old_fan_in = self.incoming.get(&old_node.id).map_or(0, |v| v.len());
                let old_fan_out = self.outgoing.get(&old_node.id).map_or(0, |v| v.len());
                let new_fan_in = other.incoming.get(&old_node.id).map_or(0, |v| v.len());
                let new_fan_out = other.outgoing.get(&old_node.id).map_or(0, |v| v.len());

                let old_total = (old_fan_in + old_fan_out) as i32;
                let new_total = (new_fan_in + new_fan_out) as i32;
                let change = new_total - old_total;

                if change != 0 {
                    complexity_changes.push(ComplexityChange {
                        node_id: old_node.id.clone(),
                        node_name: old_node.name.clone(),
                        old_fan_in,
                        new_fan_in,
                        old_fan_out,
                        new_fan_out,
                        change,
                    });
                }
            }
        }

        let added_edges_count = other.edges.len().saturating_sub(self.edges.len());
        let removed_edges_count = self.edges.len().saturating_sub(other.edges.len());

        GraphDiff {
            added_nodes,
            removed_nodes,
            changed_nodes,
            added_edges_count,
            removed_edges_count,
            complexity_changes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResult {
    pub from_id: String,
    pub to_name: String,
    pub edge_type: super::EdgeType,
    pub call_site: String,
    pub file_path: std::path::PathBuf,
    pub line: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub fan_in: usize,
    pub fan_out: usize,
    pub cyclomatic: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotResult {
    pub name: String,
    pub call_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    pub added_nodes: Vec<String>,   // Node IDs
    pub removed_nodes: Vec<String>, // Node IDs
    pub changed_nodes: Vec<NodeChange>,
    pub added_edges_count: usize,
    pub removed_edges_count: usize,
    pub complexity_changes: Vec<ComplexityChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeChange {
    pub node_id: String,
    pub node_name: String,
    pub old_signature: String,
    pub new_signature: String,
    pub old_line: usize,
    pub new_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityChange {
    pub node_id: String,
    pub node_name: String,
    pub old_fan_in: usize,
    pub new_fan_in: usize,
    pub old_fan_out: usize,
    pub new_fan_out: usize,
    pub change: i32, // positive = increased, negative = decreased
}
