use crate::core::CodeGraph;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_to_file(graph: &CodeGraph, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(graph)?;
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_from_file(input_path: &Path) -> Result<CodeGraph> {
    let file = File::open(input_path)?;
    let mut graph: CodeGraph = serde_json::from_reader(file)?;
    graph.build_indexes();
    Ok(graph)
}
