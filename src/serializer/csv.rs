use crate::core::CodeGraph;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_to_files(graph: &CodeGraph, output_prefix: &Path) -> Result<()> {
    // Generate nodes.csv and edges.csv files
    let nodes_path = output_prefix.with_file_name(format!(
        "{}_nodes.csv",
        output_prefix.file_stem().unwrap().to_string_lossy()
    ));
    let edges_path = output_prefix.with_file_name(format!(
        "{}_edges.csv",
        output_prefix.file_stem().unwrap().to_string_lossy()
    ));

    // Write nodes CSV
    let mut nodes_file = File::create(&nodes_path)?;
    writeln!(
        nodes_file,
        "id,name,type,file_path,line,end_line,package,signature"
    )?;

    for node in &graph.nodes {
        let node_type = format!("{:?}", node.node_type);
        writeln!(
            nodes_file,
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",\"{}\"",
            escape_csv(&node.id),
            escape_csv(&node.name),
            node_type,
            escape_csv(&node.file_path.display().to_string()),
            node.line,
            node.end_line,
            escape_csv(&node.package),
            escape_csv(&node.signature)
        )?;
    }

    // Write edges CSV
    let mut edges_file = File::create(&edges_path)?;
    writeln!(edges_file, "from,to,type,call_site,file_path,line")?;

    for edge in &graph.edges {
        let edge_type = format!("{:?}", edge.edge_type);
        writeln!(
            edges_file,
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}",
            escape_csv(&edge.from),
            escape_csv(&edge.to),
            edge_type,
            escape_csv(&edge.call_site),
            escape_csv(&edge.file_path.display().to_string()),
            edge.line
        )?;
    }

    println!("Nodes written to: {}", nodes_path.display());
    println!("Edges written to: {}", edges_path.display());

    Ok(())
}

fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}
