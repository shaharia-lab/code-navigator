use crate::core::CodeGraph;
use anyhow::Result;

/// Save graph to JSON format with Zstd compression (Phase 2 optimization)
/// ~5-10x faster than Gzip, better compression
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    // Serialize to JSON (respects serde attributes)
    let json = serde_json::to_vec(graph)?;

    // Compress with Zstd (level 3 = good balance of speed/compression)
    let compressed = zstd::encode_all(&json[..], 3)?;

    // Write directly to file
    std::fs::write(path, compressed)?;

    Ok(())
}

/// Load graph from JSON+Zstd format
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    // Read compressed data from file
    let compressed = std::fs::read(path)?;

    // Decompress with Zstd
    let decompressed = zstd::decode_all(&compressed[..])?;

    // Deserialize from JSON
    let mut graph: CodeGraph = serde_json::from_slice(&decompressed)?;
    graph.build_indexes();
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compressed_roundtrip() {
        let mut graph = CodeGraph::new("/test".to_string(), "typescript".to_string());

        // Add a test node
        let node = crate::core::Node {
            id: "test1".to_string(),
            name: "testFunc".to_string(),
            node_type: crate::core::NodeType::Function,
            package: "test".to_string(),
            file_path: std::path::PathBuf::from("/test/file.ts"),
            line: 10,
            end_line: 15,
            signature: "testFunc()".to_string(),
            parameters: vec![],
            returns: vec![],
            documentation: None,
            tags: vec![],
            metadata: Default::default(),
        };
        graph.add_node(node);

        // Save and load - keep temp_file in scope
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_to_file(&graph, path).unwrap();
        let loaded = load_from_file(path).unwrap();

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "testFunc");

        // Keep temp_file alive until end of test
        drop(temp_file);
    }
}
