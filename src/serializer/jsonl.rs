use crate::core::CodeGraph;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export graph to JSONL (JSON Lines) format
/// Each line is a separate JSON object for streaming processing
pub fn export_jsonl(graph: &CodeGraph, output_path: &str) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // Write metadata as first line
    let metadata_line = serde_json::json!({
        "type": "metadata",
        "version": graph.metadata.version,
        "generated_at": graph.metadata.generated_at,
        "generator": graph.metadata.generator,
        "language": graph.metadata.language,
        "root_path": graph.metadata.root_path,
        "stats": {
            "total_nodes": graph.metadata.stats.total_nodes,
            "total_edges": graph.metadata.stats.total_edges,
            "files_parsed": graph.metadata.stats.files_parsed,
        }
    });
    writeln!(writer, "{}", serde_json::to_string(&metadata_line)?)?;

    // Write each node as a line
    for node in &graph.nodes {
        let node_line = serde_json::json!({
            "type": "node",
            "id": node.id,
            "name": node.name,
            "node_type": format!("{:?}", node.node_type),
            "file_path": node.file_path.display().to_string(),
            "line": node.line,
            "end_line": node.end_line,
            "package": node.package,
            "signature": node.signature,
            "parameters": node.parameters,
            "returns": node.returns,
            "documentation": node.documentation,
            "tags": node.tags,
            "metadata": node.metadata,
        });
        writeln!(writer, "{}", serde_json::to_string(&node_line)?)?;
    }

    // Write each edge as a line
    for edge in &graph.edges {
        let edge_line = serde_json::json!({
            "type": "edge",
            "from": edge.from,
            "to": edge.to,
            "edge_type": format!("{:?}", edge.edge_type),
            "call_site": edge.call_site,
            "file_path": edge.file_path.display().to_string(),
            "line": edge.line,
            "metadata": edge.metadata,
        });
        writeln!(writer, "{}", serde_json::to_string(&edge_line)?)?;
    }

    writer.flush()?;
    Ok(())
}

/// Load graph from JSONL format
pub fn load_from_jsonl(input_path: &str) -> Result<CodeGraph> {
    use crate::core::{Edge, EdgeType, GraphMetadata, GraphStats, Node, NodeType, Parameter};
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;

    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut metadata: Option<GraphMetadata> = None;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let value: serde_json::Value = serde_json::from_str(&line)?;

        match value["type"].as_str() {
            Some("metadata") => {
                metadata = Some(GraphMetadata {
                    version: value["version"].as_str().unwrap_or("1.0.0").to_string(),
                    generated_at: value["generated_at"].as_str().unwrap_or("").to_string(),
                    generator: value["generator"]
                        .as_str()
                        .unwrap_or("code-navigator")
                        .to_string(),
                    language: value["language"].as_str().unwrap_or("").to_string(),
                    root_path: value["root_path"].as_str().unwrap_or("").to_string(),
                    stats: GraphStats {
                        total_nodes: value["stats"]["total_nodes"].as_u64().unwrap_or(0) as usize,
                        total_edges: value["stats"]["total_edges"].as_u64().unwrap_or(0) as usize,
                        files_parsed: value["stats"]["files_parsed"].as_u64().unwrap_or(0) as usize,
                    },
                    file_metadata: HashMap::new(),
                    git_commit_hash: None,
                });
            }
            Some("node") => {
                let node_type = match value["node_type"].as_str().unwrap_or("Function") {
                    "Function" => NodeType::Function,
                    "Method" => NodeType::Method,
                    "HttpHandler" => NodeType::HttpHandler,
                    "Middleware" => NodeType::Middleware,
                    _ => NodeType::Function,
                };

                let parameters: Vec<Parameter> =
                    if let Some(params_array) = value["parameters"].as_array() {
                        params_array
                            .iter()
                            .filter_map(|p| {
                                Some(Parameter {
                                    name: p["name"].as_str()?.to_string(),
                                    param_type: p["param_type"].as_str()?.to_string(),
                                })
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };

                let returns: Vec<String> = if let Some(ret_array) = value["returns"].as_array() {
                    ret_array
                        .iter()
                        .filter_map(|r| r.as_str().map(|s| s.to_string()))
                        .collect()
                } else {
                    Vec::new()
                };

                let tags: Vec<String> = if let Some(tag_array) = value["tags"].as_array() {
                    tag_array
                        .iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                } else {
                    Vec::new()
                };

                let metadata_map: std::collections::HashMap<String, String> =
                    if let Some(meta_obj) = value["metadata"].as_object() {
                        meta_obj
                            .iter()
                            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                            .collect()
                    } else {
                        std::collections::HashMap::new()
                    };

                let node = Node {
                    id: value["id"].as_str().unwrap_or("").to_string(),
                    name: value["name"].as_str().unwrap_or("").to_string(),
                    node_type,
                    file_path: PathBuf::from(value["file_path"].as_str().unwrap_or("")),
                    line: value["line"].as_u64().unwrap_or(0) as usize,
                    end_line: value["end_line"].as_u64().unwrap_or(0) as usize,
                    package: value["package"].as_str().unwrap_or("").to_string(),
                    signature: value["signature"].as_str().unwrap_or("").to_string(),
                    parameters,
                    returns,
                    documentation: value["documentation"].as_str().map(|s| s.to_string()),
                    tags,
                    metadata: metadata_map,
                };
                nodes.push(node);
            }
            Some("edge") => {
                let edge_type = match value["edge_type"].as_str().unwrap_or("Calls") {
                    "Calls" => EdgeType::Calls,
                    "Imports" => EdgeType::Imports,
                    "Implements" => EdgeType::Implements,
                    _ => EdgeType::Calls,
                };

                let metadata_map: std::collections::HashMap<String, String> =
                    if let Some(meta_obj) = value["metadata"].as_object() {
                        meta_obj
                            .iter()
                            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                            .collect()
                    } else {
                        std::collections::HashMap::new()
                    };

                let edge = Edge {
                    from: value["from"].as_str().unwrap_or("").to_string(),
                    to: value["to"].as_str().unwrap_or("").to_string(),
                    edge_type,
                    call_site: value["call_site"].as_str().unwrap_or("").to_string(),
                    file_path: PathBuf::from(value["file_path"].as_str().unwrap_or("")),
                    line: value["line"].as_u64().unwrap_or(0) as usize,
                    metadata: metadata_map,
                };
                edges.push(edge);
            }
            _ => {
                // Unknown type, skip
            }
        }
    }

    let metadata = metadata.unwrap_or_else(|| GraphMetadata {
        version: "1.0.0".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        generator: "code-navigator".to_string(),
        language: "unknown".to_string(),
        root_path: "".to_string(),
        stats: GraphStats {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            files_parsed: 0,
        },
        file_metadata: HashMap::new(),
        git_commit_hash: None,
    });

    let mut graph = CodeGraph {
        metadata,
        nodes,
        edges,
        node_by_id: Default::default(),
        outgoing: Default::default(),
        incoming: Default::default(),
        by_name: Default::default(),
        by_type: Default::default(),
        indices_dirty: true,
    };

    graph.build_indexes();
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Edge, EdgeType, GraphMetadata, GraphStats, Node, NodeType};
    use std::path::PathBuf;

    #[test]
    fn test_jsonl_roundtrip() {
        let mut graph = CodeGraph {
            metadata: GraphMetadata {
                version: "1.0.0".to_string(),
                generated_at: "2026-01-31T00:00:00Z".to_string(),
                generator: "test".to_string(),
                language: "go".to_string(),
                root_path: "/test".to_string(),
                stats: GraphStats {
                    total_nodes: 1,
                    total_edges: 1,
                    files_parsed: 1,
                },
                file_metadata: HashMap::new(),
                git_commit_hash: None,
            },
            nodes: vec![Node {
                id: "test:func1:10".to_string(),
                name: "func1".to_string(),
                node_type: NodeType::Function,
                file_path: PathBuf::from("test.go"),
                line: 10,
                end_line: 20,
                package: "main".to_string(),
                signature: "func func1()".to_string(),
                parameters: vec![],
                returns: vec![],
                documentation: None,
                tags: vec![],
                metadata: Default::default(),
            }],
            edges: vec![Edge {
                from: "test:func1:10".to_string(),
                to: "func2".to_string(),
                edge_type: EdgeType::Calls,
                call_site: "func2()".to_string(),
                file_path: PathBuf::from("test.go"),
                line: 15,
                metadata: Default::default(),
            }],
            node_by_id: Default::default(),
            outgoing: Default::default(),
            incoming: Default::default(),
            by_name: Default::default(),
            by_type: Default::default(),
            indices_dirty: true,
        };

        graph.build_indexes();

        // Export to JSONL
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        export_jsonl(&graph, temp_path).unwrap();

        // Load from JSONL
        let loaded_graph = load_from_jsonl(temp_path).unwrap();

        assert_eq!(loaded_graph.nodes.len(), 1);
        assert_eq!(loaded_graph.edges.len(), 1);
        assert_eq!(loaded_graph.nodes[0].name, "func1");
        assert_eq!(loaded_graph.edges[0].to, "func2");
    }
}
