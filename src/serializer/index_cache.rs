use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::NodeType;

/// Serialized indices for fast loading
/// Stored as a companion .idx file alongside the graph binary
#[derive(Serialize, Deserialize)]
pub struct SerializedIndices {
    /// Version string for compatibility checking
    pub version: String,

    /// Hash of the graph structure for validation
    pub graph_hash: String,

    /// Total nodes count for validation
    pub node_count: usize,

    /// Total edges count for validation
    pub edge_count: usize,

    /// Node ID → node index
    pub node_by_id: HashMap<String, usize>,

    /// Node name → node indices (names can be duplicated)
    pub by_name: HashMap<String, Vec<usize>>,

    /// Node type → node indices
    pub by_type: HashMap<NodeType, Vec<usize>>,

    /// Edge source node ID → edge indices
    pub outgoing: HashMap<String, Vec<usize>>,

    /// Edge target node ID → edge indices
    pub incoming: HashMap<String, Vec<usize>>,
}

impl SerializedIndices {
    /// Create from current graph indices
    pub fn from_graph(
        node_count: usize,
        edge_count: usize,
        graph_hash: String,
        node_by_id: &HashMap<String, usize>,
        by_name: &HashMap<String, Vec<usize>>,
        by_type: &HashMap<NodeType, Vec<usize>>,
        outgoing: &HashMap<String, Vec<usize>>,
        incoming: &HashMap<String, Vec<usize>>,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            graph_hash,
            node_count,
            edge_count,
            node_by_id: node_by_id.clone(),
            by_name: by_name.clone(),
            by_type: by_type.clone(),
            outgoing: outgoing.clone(),
            incoming: incoming.clone(),
        }
    }

    /// Save serialized indices to disk with compression
    pub fn save(&self, graph_path: &Path) -> Result<()> {
        let idx_path = graph_path.with_extension("idx");

        // Serialize with bincode (faster than JSON)
        let data = bincode::serialize(self)?;

        // Compress with zstd (fast compression level)
        let compressed = zstd::encode_all(&data[..], 1)?;

        std::fs::write(idx_path, compressed)?;
        Ok(())
    }

    /// Load serialized indices from disk
    pub fn load(graph_path: &Path) -> Result<Self> {
        let idx_path = graph_path.with_extension("idx");

        if !idx_path.exists() {
            anyhow::bail!("Index cache file not found");
        }

        let compressed = std::fs::read(idx_path)?;
        let data = zstd::decode_all(&compressed[..])?;
        let indices: Self = bincode::deserialize(&data)?;

        Ok(indices)
    }

    /// Validate that cached indices match the current graph
    pub fn validate(&self, node_count: usize, edge_count: usize, graph_hash: &str) -> bool {
        self.node_count == node_count
            && self.edge_count == edge_count
            && self.graph_hash == graph_hash
            && self.version == env!("CARGO_PKG_VERSION")
    }
}
