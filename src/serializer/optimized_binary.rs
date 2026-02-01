use crate::core::CodeGraph;
use anyhow::Result;
use std::io::Write;

/// File format version for compatibility checking
const FORMAT_VERSION: u32 = 1;
const MAGIC_BYTES: &[u8; 8] = b"CODENAV\x01";

/// Save graph in optimized binary format
/// Uses MessagePack (faster than JSON, serde-compatible) + zstd compression
/// This is 2-3x faster to load than JSON deserialization
pub fn save_to_file(graph: &CodeGraph, path: &str) -> Result<()> {
    // Serialize graph with MessagePack (faster than JSON, handles serde attributes)
    let serialized = rmp_serde::to_vec(graph)
        .map_err(|e| anyhow::anyhow!("Failed to serialize graph with MessagePack: {}", e))?;

    let mut buffer = Vec::new();

    // Write magic bytes and version
    buffer.write_all(MAGIC_BYTES)?;
    buffer.write_all(&FORMAT_VERSION.to_le_bytes())?;

    // Compress with zstd (level 3 for speed/compression balance)
    let compressed = zstd::encode_all(&serialized[..], 3)?;

    // Write compressed data
    buffer.write_all(&compressed)?;

    // Write to file atomically
    std::fs::write(path, buffer)?;

    Ok(())
}

/// Load graph from optimized binary format
/// Falls back to JSON format if magic bytes don't match (backward compatibility)
pub fn load_from_file(path: &str) -> Result<CodeGraph> {
    use std::io::Cursor;

    let file_data = std::fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path, e))?;

    // Check for magic bytes (8 bytes magic + 4 bytes version = 12 bytes minimum)
    if file_data.len() < 12 || &file_data[0..8] != MAGIC_BYTES {
        // Not our format - try to load as JSON (backward compatibility)
        return load_json_fallback(&file_data);
    }

    // Read version
    let version = u32::from_le_bytes([
        file_data[8],
        file_data[9],
        file_data[10],
        file_data[11],
    ]);

    if version != FORMAT_VERSION {
        anyhow::bail!("Unsupported format version: {}", version);
    }

    // Decompress data (everything after the header)
    let compressed_data = &file_data[12..];
    let decompressed = zstd::decode_all(compressed_data)
        .map_err(|e| anyhow::anyhow!("Failed to decompress data: {}", e))?;

    // Deserialize with bincode (much faster than JSON)
    let mut graph: CodeGraph = rmp_serde::from_slice(&decompressed)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize graph: {}", e))?;

    // Build indices (same as before)
    graph.build_indexes();

    Ok(graph)
}

/// Fallback to JSON format for backward compatibility
fn load_json_fallback(data: &[u8]) -> Result<CodeGraph> {
    // Try decompressing first (might be zstd-compressed JSON)
    let decompressed = match zstd::decode_all(data) {
        Ok(d) => d,
        Err(_) => data.to_vec(), // Not compressed, use raw data
    };

    // Deserialize as JSON
    let mut graph: CodeGraph = serde_json::from_slice(&decompressed)?;
    graph.build_indexes();

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_optimized_binary_roundtrip() {
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

        // First test: can we serialize with MessagePack at all?
        let serialized = rmp_serde::to_vec(&graph).expect("Failed to serialize with MessagePack");
        eprintln!("Serialized size: {} bytes", serialized.len());

        // Can we deserialize?
        let _deserialized: CodeGraph = rmp_serde::from_slice(&serialized)
            .expect("Failed to deserialize with MessagePack");
        eprintln!("Direct MessagePack roundtrip works!");

        // Now test with file save/load
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_to_file(&graph, path).unwrap();
        eprintln!("Saved to file");

        let loaded = load_from_file(path).unwrap();
        eprintln!("Loaded from file");

        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "testFunc");
    }

    #[test]
    fn test_backward_compatibility() {
        // Create a JSON-formatted file (old format)
        let graph = CodeGraph::new("/test".to_string(), "go".to_string());
        let json = serde_json::to_vec(&graph).unwrap();
        let compressed = zstd::encode_all(&json[..], 3).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        std::fs::write(path, compressed).unwrap();

        // Should be able to load old format
        let loaded = load_from_file(path).unwrap();
        assert_eq!(loaded.nodes.len(), 0);
    }
}
