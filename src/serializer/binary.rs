use crate::core::CodeGraph;
use anyhow::Result;

/// Save graph to binary format (uses compressed JSON internally for stability)
/// Much faster than plain JSON and produces smaller files
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    // Use compressed JSON for stability (bincode has issues with serde(skip) fields)
    crate::serializer::compressed::save_to_file(graph, path)
}

/// Load graph from binary format
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    crate::serializer::compressed::load_from_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_binary_roundtrip() {
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

        // Check file exists and has content
        let metadata = std::fs::metadata(path).unwrap();
        assert!(metadata.len() > 0, "File should not be empty");

        let loaded = load_from_file(path).unwrap();

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "testFunc");

        // Keep temp_file alive until end of test
        drop(temp_file);
    }
}
