use crate::core::CodeGraph;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_to_file(graph: &CodeGraph, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write DOT header
    writeln!(file, "digraph CodeGraph {{")?;
    writeln!(file, "  rankdir=LR;")?;
    writeln!(file, "  node [shape=box];")?;
    writeln!(file)?;

    // Write nodes
    for node in &graph.nodes {
        let node_type = format!("{:?}", node.node_type);
        let label = format!(
            "{}\\n{}\\n{}:{}",
            node.name, node_type, node.package, node.line
        );

        // Color nodes by type
        let color = match node.node_type {
            crate::core::NodeType::Function => "lightblue",
            crate::core::NodeType::Method => "lightgreen",
            crate::core::NodeType::HttpHandler => "yellow",
            crate::core::NodeType::Middleware => "pink",
        };

        writeln!(
            file,
            "  \"{}\" [label=\"{}\", fillcolor={}, style=filled];",
            escape_dot(&node.id),
            escape_dot(&label),
            color
        )?;
    }

    writeln!(file)?;

    // Write edges
    for edge in &graph.edges {
        let edge_type = format!("{:?}", edge.edge_type);

        // Try to find the target node to link to its ID
        // If not found, just use the function name
        let target = if let Some(_target_node) = graph.get_nodes_by_name(&edge.to).first() {
            format!("{}:{}", edge.to, edge.line)
        } else {
            edge.to.clone()
        };

        writeln!(
            file,
            "  \"{}\" -> \"{}\" [label=\"{}\"];",
            escape_dot(&edge.from),
            escape_dot(&target),
            escape_dot(&edge_type)
        )?;
    }

    // Close digraph
    writeln!(file, "}}")?;

    Ok(())
}

fn escape_dot(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
