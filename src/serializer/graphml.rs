use crate::core::CodeGraph;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_to_file(graph: &CodeGraph, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write GraphML header
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(
        file,
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\""
    )?;
    writeln!(
        file,
        "         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""
    )?;
    writeln!(
        file,
        "         xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns"
    )?;
    writeln!(
        file,
        "         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">"
    )?;
    writeln!(file)?;

    // Define attributes
    writeln!(
        file,
        "  <key id=\"d0\" for=\"node\" attr.name=\"name\" attr.type=\"string\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d1\" for=\"node\" attr.name=\"type\" attr.type=\"string\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d2\" for=\"node\" attr.name=\"file\" attr.type=\"string\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d3\" for=\"node\" attr.name=\"line\" attr.type=\"int\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d4\" for=\"node\" attr.name=\"package\" attr.type=\"string\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d5\" for=\"edge\" attr.name=\"type\" attr.type=\"string\"/>"
    )?;
    writeln!(
        file,
        "  <key id=\"d6\" for=\"edge\" attr.name=\"call_site\" attr.type=\"string\"/>"
    )?;
    writeln!(file)?;

    // Start graph
    writeln!(file, "  <graph id=\"G\" edgedefault=\"directed\">")?;

    // Write nodes
    for node in &graph.nodes {
        let node_type = format!("{:?}", node.node_type);
        let file_path = node.file_path.display().to_string();

        writeln!(file, "    <node id=\"{}\">", escape_xml(&node.id))?;
        writeln!(
            file,
            "      <data key=\"d0\">{}</data>",
            escape_xml(&node.name)
        )?;
        writeln!(
            file,
            "      <data key=\"d1\">{}</data>",
            escape_xml(&node_type)
        )?;
        writeln!(
            file,
            "      <data key=\"d2\">{}</data>",
            escape_xml(&file_path)
        )?;
        writeln!(file, "      <data key=\"d3\">{}</data>", node.line)?;
        writeln!(
            file,
            "      <data key=\"d4\">{}</data>",
            escape_xml(&node.package)
        )?;
        writeln!(file, "    </node>")?;
    }

    // Write edges
    for (idx, edge) in graph.edges.iter().enumerate() {
        let edge_type = format!("{:?}", edge.edge_type);

        writeln!(
            file,
            "    <edge id=\"e{}\" source=\"{}\" target=\"{}\">",
            idx,
            escape_xml(&edge.from),
            escape_xml(&edge.to)
        )?;
        writeln!(
            file,
            "      <data key=\"d5\">{}</data>",
            escape_xml(&edge_type)
        )?;
        writeln!(
            file,
            "      <data key=\"d6\">{}</data>",
            escape_xml(&edge.call_site)
        )?;
        writeln!(file, "    </edge>")?;
    }

    // Close graph and graphml
    writeln!(file, "  </graph>")?;
    writeln!(file, "</graphml>")?;

    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
