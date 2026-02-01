pub mod core;
pub mod parser;
pub mod serializer;

#[cfg(test)]
mod tests {
    use crate::core::{CodeGraph, Edge, EdgeType, Node, NodeType};
    use std::path::PathBuf;

    #[test]
    fn test_create_node() {
        let node = Node::new(
            "test:func:1".to_string(),
            "TestFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func TestFunc() {}".to_string(),
        );

        assert_eq!(node.name, "TestFunc");
        assert_eq!(node.line, 1);
        assert_eq!(node.end_line, 10);
        assert_eq!(node.package, "main");
    }

    #[test]
    fn test_create_edge() {
        let edge = Edge::new(
            "from:func:1".to_string(),
            "ToFunc".to_string(),
            EdgeType::Calls,
            "ToFunc()".to_string(),
            PathBuf::from("test.go"),
            5,
        );

        assert_eq!(edge.from, "from:func:1");
        assert_eq!(edge.to, "ToFunc");
        assert_eq!(edge.line, 5);
    }

    #[test]
    fn test_graph_add_node() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        let node = Node::new(
            "test:func:1".to_string(),
            "TestFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func TestFunc() {}".to_string(),
        );

        graph.add_node(node);

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.metadata.stats.total_nodes, 1);
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        let edge = Edge::new(
            "from:func:1".to_string(),
            "ToFunc".to_string(),
            EdgeType::Calls,
            "ToFunc()".to_string(),
            PathBuf::from("test.go"),
            5,
        );

        graph.add_edge(edge);

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.metadata.stats.total_edges, 1);
    }

    #[test]
    fn test_graph_get_node_by_id() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        let node = Node::new(
            "test:func:1".to_string(),
            "TestFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func TestFunc() {}".to_string(),
        );

        graph.add_node(node);

        let found = graph.get_node_by_id("test:func:1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "TestFunc");
    }

    #[test]
    fn test_graph_get_nodes_by_name() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        let node = Node::new(
            "test:func:1".to_string(),
            "TestFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func TestFunc() {}".to_string(),
        );

        graph.add_node(node);

        let nodes = graph.get_nodes_by_name("TestFunc");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "TestFunc");
    }

    #[test]
    fn test_graph_get_nodes_by_type() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        let func_node = Node::new(
            "test:func:1".to_string(),
            "TestFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func TestFunc() {}".to_string(),
        );

        let method_node = Node::new(
            "test:method:15".to_string(),
            "TestMethod".to_string(),
            NodeType::Method,
            PathBuf::from("test.go"),
            15,
            25,
            "main".to_string(),
            "func (t *Test) TestMethod() {}".to_string(),
        );

        graph.add_node(func_node);
        graph.add_node(method_node);

        let functions = graph.get_nodes_by_type(&NodeType::Function);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "TestFunc");

        let methods = graph.get_nodes_by_type(&NodeType::Method);
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].name, "TestMethod");
    }
}
