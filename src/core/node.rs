use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Function,
    Method,
    HttpHandler,
    Middleware,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub file_path: PathBuf,
    pub line: usize,
    pub end_line: usize,
    pub package: String,
    pub signature: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub returns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Node {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        node_type: NodeType,
        file_path: PathBuf,
        line: usize,
        end_line: usize,
        package: String,
        signature: String,
    ) -> Self {
        Self {
            id,
            name,
            node_type,
            file_path,
            line,
            end_line,
            package,
            signature,
            parameters: Vec::new(),
            returns: Vec::new(),
            documentation: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}
