use crate::core::CodeGraph;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// Save graph to binary format with Zstd compression (Phase 2 optimization)
/// ~5-10x faster than JSON+Gzip, 50%+ smaller files
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Serialize to binary with bincode
    let encoded = bincode::serialize(graph)?;

    // Compress with Zstd (level 3 = good balance of speed/compression)
    let compressed = zstd::encode_all(&encoded[..], 3)?;
    std::io::Write::write_all(&mut writer, &compressed)?;

    Ok(())
}

/// Load graph from binary Zstd format
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Decompress with Zstd
    let decompressed = zstd::decode_all(reader)?;

    // Deserialize from binary
    let mut graph: CodeGraph = bincode::deserialize(&decompressed)?;
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

        // Save and load
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_to_file(&graph, path).unwrap();
        let loaded = load_from_file(path).unwrap();

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "testFunc");
    }
}
