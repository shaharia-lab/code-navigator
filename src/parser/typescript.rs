use crate::core::{CodeGraph, Edge, EdgeType, Node, NodeType, Parameter};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::Parser;

pub struct TypeScriptParser {
    parser: Parser,
    language: Language,
}

#[derive(Clone, Copy)]
pub enum Language {
    TypeScript,
    JavaScript,
}

impl TypeScriptParser {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        let ts_language = match language {
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            Language::JavaScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT, // Same parser, different extensions
        };

        parser
            .set_language(&ts_language.into())
            .context("Failed to set TypeScript language")?;
        Ok(Self { parser, language })
    }

    pub fn parse_directory(&mut self, dir: &Path, graph: &mut CodeGraph) -> Result<()> {
        use rayon::prelude::*;

        let extensions = match self.language {
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx"],
        };

        // Collect all file paths first
        let file_paths: Vec<_> = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Some(ext) = e.path().extension().and_then(|s| s.to_str()) {
                    extensions.contains(&ext)
                        && !e.path().to_string_lossy().contains(".test.")
                        && !e.path().to_string_lossy().contains(".spec.")
                } else {
                    false
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parse files in parallel
        let language = self.language;
        let results: Vec<CodeGraph> = file_paths
            .par_iter()
            .filter_map(|path| {
                // Each thread gets its own parser
                let mut parser = match Self::new(language) {
                    Ok(p) => p,
                    Err(_) => return None,
                };

                let mut temp_graph = CodeGraph::new(
                    dir.to_string_lossy().to_string(),
                    match language {
                        Language::TypeScript => "typescript".to_string(),
                        Language::JavaScript => "javascript".to_string(),
                    },
                );

                match parser.parse_file(path, &mut temp_graph) {
                    Ok(()) => Some(temp_graph),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        // Merge all results
        let files_parsed = results.len();
        for temp_graph in results {
            graph.merge(temp_graph);
        }

        graph.metadata.stats.files_parsed = files_parsed;
        graph.metadata.stats.total_nodes = graph.nodes.len();
        graph.metadata.stats.total_edges = graph.edges.len();
        graph.build_indexes();
        Ok(())
    }

    pub fn parse_file(&mut self, file_path: &Path, graph: &mut CodeGraph) -> Result<()> {
        let source = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {}", file_path.display()))?;

        let tree = self
            .parser
            .parse(&source, None)
            .context("Failed to parse TypeScript file")?;

        let root = tree.root_node();

        // For TypeScript/JavaScript, we'll use "module" as the default package
        let package_name = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string();

        // Walk the tree to extract functions and methods
        self.walk_tree(root, &source, file_path, &package_name, graph)?;

        Ok(())
    }

    fn walk_tree(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        package_name: &str,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        match node.kind() {
            "function_declaration" | "function" => {
                self.extract_function(node, source, file_path, package_name, graph)?;
            }
            "method_definition" => {
                self.extract_method(node, source, file_path, package_name, graph)?;
            }
            "arrow_function" => {
                self.extract_arrow_function(node, source, file_path, package_name, graph)?;
            }
            "class_declaration" => {
                // For classes, we still want to traverse to find methods
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_tree(child, source, file_path, package_name, graph)?;
        }

        Ok(())
    }

    fn extract_function(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        package_name: &str,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        let mut func_name = String::new();
        let mut parameters = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" if func_name.is_empty() => {
                    func_name = source[child.byte_range()].to_string();
                }
                "formal_parameters" => {
                    parameters = self.extract_parameters(child, source);
                }
                _ => {}
            }
        }

        if func_name.is_empty() {
            func_name = "anonymous".to_string();
        }

        let line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        let signature = source[node.byte_range()]
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        let id = format!("{}:{}:{}", file_path.display(), func_name, line);

        let mut node_obj = Node::new(
            id,
            func_name.clone(),
            NodeType::Function,
            file_path.to_path_buf(),
            line,
            end_line,
            package_name.to_string(),
            signature,
        );
        node_obj.parameters = parameters;
        graph.add_node(node_obj);

        // Extract calls within this function
        self.extract_calls_in_node(node, source, file_path, &func_name, line, graph)?;

        Ok(())
    }

    fn extract_method(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        package_name: &str,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        let mut method_name = String::new();
        let mut parameters = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "property_identifier" if method_name.is_empty() => {
                    method_name = source[child.byte_range()].to_string();
                }
                "formal_parameters" => {
                    parameters = self.extract_parameters(child, source);
                }
                _ => {}
            }
        }

        if !method_name.is_empty() {
            let line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            let signature = source[node.byte_range()]
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let id = format!("{}:{}:{}", file_path.display(), method_name, line);

            let mut node_obj = Node::new(
                id,
                method_name.clone(),
                NodeType::Method,
                file_path.to_path_buf(),
                line,
                end_line,
                package_name.to_string(),
                signature,
            );
            node_obj.parameters = parameters;
            graph.add_node(node_obj);

            // Extract calls within this method
            self.extract_calls_in_node(node, source, file_path, &method_name, line, graph)?;
        }

        Ok(())
    }

    fn extract_arrow_function(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        package_name: &str,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        // Try to find if this arrow function is assigned to a variable
        let parent = node.parent();
        let mut func_name = String::new();

        if let Some(parent_node) = parent {
            if parent_node.kind() == "variable_declarator" {
                let mut cursor = parent_node.walk();
                for child in parent_node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        func_name = source[child.byte_range()].to_string();
                        break;
                    }
                }
            }
        }

        if func_name.is_empty() {
            // Skip anonymous arrow functions that aren't assigned
            return Ok(());
        }

        let mut parameters = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "formal_parameters" {
                parameters = self.extract_parameters(child, source);
            }
        }

        let line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        let signature = source[node.byte_range()]
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        let id = format!("{}:{}:{}", file_path.display(), func_name, line);

        let mut node_obj = Node::new(
            id,
            func_name.clone(),
            NodeType::Function,
            file_path.to_path_buf(),
            line,
            end_line,
            package_name.to_string(),
            signature,
        );
        node_obj.parameters = parameters;
        graph.add_node(node_obj);

        // Extract calls within this arrow function
        self.extract_calls_in_node(node, source, file_path, &func_name, line, graph)?;

        Ok(())
    }

    fn extract_parameters(&self, node: tree_sitter::Node, source: &str) -> Vec<Parameter> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    let mut name = String::new();
                    let mut param_type = String::new();

                    let mut param_cursor = child.walk();
                    for param_child in child.children(&mut param_cursor) {
                        match param_child.kind() {
                            "identifier" if name.is_empty() => {
                                name = source[param_child.byte_range()].to_string();
                            }
                            "type_annotation" => {
                                // Extract type from type annotation
                                let mut type_cursor = param_child.walk();
                                for type_child in param_child.children(&mut type_cursor) {
                                    if type_child.kind() != ":" {
                                        param_type = source[type_child.byte_range()].to_string();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if !name.is_empty() {
                        parameters.push(Parameter {
                            name,
                            param_type: if param_type.is_empty() {
                                "any".to_string()
                            } else {
                                param_type
                            },
                        });
                    }
                }
                _ => {}
            }
        }

        parameters
    }

    fn extract_calls_in_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        func_name: &str,
        func_line: usize,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        self.find_calls(node, source, file_path, func_name, func_line, graph);
        Ok(())
    }

    fn find_calls(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        func_name: &str,
        func_line: usize,
        graph: &mut CodeGraph,
    ) {
        if node.kind() == "call_expression" {
            let mut called_func = String::new();
            let mut cursor = node.walk();

            for child in node.children(&mut cursor) {
                match child.kind() {
                    "identifier" => {
                        called_func = source[child.byte_range()].to_string();
                    }
                    "member_expression" => {
                        // For method calls like obj.method()
                        let mut member_cursor = child.walk();
                        for member_child in child.children(&mut member_cursor) {
                            if member_child.kind() == "property_identifier" {
                                called_func = source[member_child.byte_range()].to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !called_func.is_empty() {
                let line = node.start_position().row + 1;
                let call_site = source[node.byte_range()].to_string();
                let from_id = format!("{}:{}:{}", file_path.display(), func_name, func_line);

                if graph.get_node_by_id(&from_id).is_some() {
                    let edge = Edge::new(
                        from_id,
                        called_func,
                        EdgeType::Calls,
                        call_site,
                        file_path.to_path_buf(),
                        line,
                    );
                    graph.add_edge(edge);
                }
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_calls(child, source, file_path, func_name, func_line, graph);
        }
    }
}
