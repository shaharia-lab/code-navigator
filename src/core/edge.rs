use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Calls,
    Imports,
    Implements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    pub call_site: String,
    pub file_path: PathBuf,
    pub line: usize,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Edge {
    pub fn new(
        from: String,
        to: String,
        edge_type: EdgeType,
        call_site: String,
        file_path: PathBuf,
        line: usize,
    ) -> Self {
        Self {
            from,
            to,
            edge_type,
            call_site,
            file_path,
            line,
            metadata: HashMap::new(),
        }
    }
}
