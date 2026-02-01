use crate::core::{CodeGraph, Edge, EdgeType, Node, NodeType, Parameter};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::Parser;

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .context("Failed to set Python language")?;
        Ok(Self { parser })
    }

    pub fn parse_directory(&mut self, dir: &Path, graph: &mut CodeGraph) -> Result<()> {
        use rayon::prelude::*;

        // Collect all file paths first
        let file_paths: Vec<_> = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("py")
                    && !e.path().to_string_lossy().contains("_test.py")
                    && !e.path().to_string_lossy().contains("test_")
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parse files in parallel
        let results: Vec<CodeGraph> = file_paths
            .par_iter()
            .filter_map(|path| {
                // Each thread gets its own parser
                let mut parser = match Self::new() {
                    Ok(p) => p,
                    Err(_) => return None,
                };

                let mut temp_graph =
                    CodeGraph::new(dir.to_string_lossy().to_string(), "python".to_string());

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
            .context("Failed to parse Python file")?;

        let root = tree.root_node();

        // For Python, use the module/directory name as package
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
        if node.kind() == "function_definition" {
            // Check if it's inside a class (method) or standalone (function)
            if self.is_inside_class(node) {
                self.extract_method(node, source, file_path, package_name, graph)?;
            } else {
                self.extract_function(node, source, file_path, package_name, graph)?;
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_tree(child, source, file_path, package_name, graph)?;
        }

        Ok(())
    }

    fn is_inside_class(&self, node: tree_sitter::Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "class_definition" {
                return true;
            }
            current = parent.parent();
        }
        false
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
                "parameters" => {
                    parameters = self.extract_parameters(child, source);
                }
                _ => {}
            }
        }

        if !func_name.is_empty() {
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
        }

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
                "identifier" if method_name.is_empty() => {
                    method_name = source[child.byte_range()].to_string();
                }
                "parameters" => {
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

    fn extract_parameters(&self, node: tree_sitter::Node, source: &str) -> Vec<Parameter> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let name = source[child.byte_range()].to_string();
                    // Skip 'self' parameter
                    if name != "self" && name != "cls" {
                        parameters.push(Parameter {
                            name,
                            param_type: "Any".to_string(),
                        });
                    }
                }
                "typed_parameter" | "default_parameter" => {
                    let mut param_name = String::new();
                    let mut param_type = String::new();

                    let mut param_cursor = child.walk();
                    for param_child in child.children(&mut param_cursor) {
                        match param_child.kind() {
                            "identifier" if param_name.is_empty() => {
                                param_name = source[param_child.byte_range()].to_string();
                            }
                            "type" => {
                                param_type = source[param_child.byte_range()].to_string();
                            }
                            _ => {}
                        }
                    }

                    if !param_name.is_empty() && param_name != "self" && param_name != "cls" {
                        parameters.push(Parameter {
                            name: param_name,
                            param_type: if param_type.is_empty() {
                                "Any".to_string()
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
        if node.kind() == "call" {
            let mut called_func = String::new();
            let mut cursor = node.walk();

            for child in node.children(&mut cursor) {
                match child.kind() {
                    "identifier" => {
                        called_func = source[child.byte_range()].to_string();
                    }
                    "attribute" => {
                        // For method calls like obj.method()
                        let mut attr_cursor = child.walk();
                        for attr_child in child.children(&mut attr_cursor) {
                            if attr_child.kind() == "identifier" {
                                // Get the last identifier (the method name)
                                called_func = source[attr_child.byte_range()].to_string();
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
