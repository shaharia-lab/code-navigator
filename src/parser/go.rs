use crate::core::{CodeGraph, Edge, EdgeType, Node, NodeType, Parameter};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::Parser;

pub struct GoParser {
    parser: Parser,
}

impl GoParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .context("Failed to set Go language")?;
        Ok(Self { parser })
    }

    pub fn parse_directory(&mut self, dir: &Path, graph: &mut CodeGraph) -> Result<()> {
        use rayon::prelude::*;

        // Phase 3: Parallel file discovery with jwalk
        let file_paths: Vec<_> = jwalk::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("go")
                    && !e.path().to_string_lossy().contains("_test.go")
            })
            .map(|e| e.path())
            .collect();

        let dir_str = dir.to_string_lossy().to_string();

        // Phase 3: Batched parallel processing for better CPU utilization
        let chunk_size = 100.min(file_paths.len().max(1));
        let results: Vec<CodeGraph> = file_paths
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut chunk_graph = CodeGraph::new_with_capacity(
                    dir_str.clone(),
                    "go".to_string(),
                    chunk.len() * 20,  // Estimate ~20 nodes per file
                    chunk.len() * 80,  // Estimate ~80 edges per file
                );

                for path in chunk {
                    let mut parser = match Self::new() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    if let Err(e) = parser.parse_file(path, &mut chunk_graph) {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }

                chunk_graph
            })
            .collect();

        // Merge all chunk results - uses incremental index updates
        let files_parsed = file_paths.len();
        for chunk_graph in results {
            graph.merge(chunk_graph);
        }

        graph.metadata.stats.files_parsed = files_parsed;
        graph.metadata.stats.total_nodes = graph.nodes.len();
        graph.metadata.stats.total_edges = graph.edges.len();
        Ok(())
    }

    pub fn parse_file(&mut self, file_path: &Path, graph: &mut CodeGraph) -> Result<()> {
        let source = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {}", file_path.display()))?;

        let tree = self
            .parser
            .parse(&source, None)
            .context("Failed to parse Go file")?;

        let root = tree.root_node();
        let package_name = self.extract_package(root, &source);

        // Walk the tree to extract functions and methods
        self.walk_tree(root, &source, file_path, &package_name, graph)?;

        Ok(())
    }

    fn extract_package(&self, node: tree_sitter::Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "package_clause" {
                let mut pkg_cursor = child.walk();
                for pkg_child in child.children(&mut pkg_cursor) {
                    if pkg_child.kind() == "package_identifier" {
                        return source[pkg_child.byte_range()].to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    fn walk_tree(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
        package_name: &str,
        graph: &mut CodeGraph,
    ) -> Result<()> {
        if node.kind() == "function_declaration" {
            self.extract_function(node, source, file_path, package_name, graph)?;
        } else if node.kind() == "method_declaration" {
            self.extract_method(node, source, file_path, package_name, graph)?;
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
                "parameter_list" => {
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
                "field_identifier" if method_name.is_empty() => {
                    method_name = source[child.byte_range()].to_string();
                }
                "parameter_list" if !parameters.is_empty() => {
                    // Second parameter_list is the method parameters (first is receiver)
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
            if child.kind() == "parameter_declaration" {
                let mut name = String::new();
                let mut param_type = String::new();

                let mut param_cursor = child.walk();
                for param_child in child.children(&mut param_cursor) {
                    match param_child.kind() {
                        "identifier" if name.is_empty() => {
                            name = source[param_child.byte_range()].to_string();
                        }
                        "type_identifier" | "pointer_type" | "slice_type" | "array_type"
                        | "map_type" | "interface_type" | "qualified_type" => {
                            param_type = source[param_child.byte_range()].to_string();
                        }
                        _ => {}
                    }
                }

                if !name.is_empty() || !param_type.is_empty() {
                    parameters.push(Parameter {
                        name: if name.is_empty() {
                            "_".to_string()
                        } else {
                            name
                        },
                        param_type,
                    });
                }
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
                    "selector_expression" => {
                        // For method calls like obj.Method()
                        let mut sel_cursor = child.walk();
                        for sel_child in child.children(&mut sel_cursor) {
                            if sel_child.kind() == "field_identifier" {
                                called_func = source[sel_child.byte_range()].to_string();
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
