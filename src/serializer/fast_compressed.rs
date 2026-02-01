use crate::core::CodeGraph;
use anyhow::Result;

/// Save graph to JSON format with LZ4 compression
/// LZ4 is 3-4x faster to decompress than zstd, with slightly larger files
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    // Serialize to JSON (respects serde attributes)
    let json = serde_json::to_vec(graph)?;

    // Compress with LZ4 (much faster decompression than zstd)
    let compressed = lz4_flex::compress_prepend_size(&json);

    // Write directly to file
    std::fs::write(path, compressed)?;

    Ok(())
}

/// Load graph from JSON+LZ4 format
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    // Read compressed data from file
    let compressed = std::fs::read(path)?;

    // Decompress with LZ4 (very fast)
    let decompressed = lz4_flex::decompress_size_prepended(&compressed)
        .map_err(|e| anyhow::anyhow!("Failed to decompress: {}", e))?;

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
    fn test_fast_compressed_roundtrip() {
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

        // Save and load
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_to_file(&graph, path).unwrap();
        let loaded = load_from_file(path).unwrap();

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "testFunc");
    }
}
