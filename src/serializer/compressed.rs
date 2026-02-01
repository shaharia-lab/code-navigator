use crate::core::CodeGraph;
use anyhow::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// Save graph to compressed JSON format (.json.gz)
/// Much smaller than JSON, slightly slower but safe
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    let file = File::create(path)?;
    let encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
    serde_json::to_writer(encoder, graph)?;
    Ok(())
}

/// Load graph from compressed JSON format
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut graph: CodeGraph = serde_json::from_reader(decoder)?;
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
