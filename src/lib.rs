pub mod benchmark;
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

    // Helper function to create a test graph with a call chain
    fn create_test_graph_with_calls() -> CodeGraph {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        // Create nodes: A -> B -> C -> D
        let node_a = Node::new(
            "test:a:1".to_string(),
            "funcA".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            5,
            "main".to_string(),
            "func funcA() {}".to_string(),
        );

        let node_b = Node::new(
            "test:b:10".to_string(),
            "funcB".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            10,
            15,
            "main".to_string(),
            "func funcB() {}".to_string(),
        );

        let node_c = Node::new(
            "test:c:20".to_string(),
            "funcC".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            20,
            25,
            "main".to_string(),
            "func funcC() {}".to_string(),
        );

        let node_d = Node::new(
            "test:d:30".to_string(),
            "funcD".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            30,
            35,
            "main".to_string(),
            "func funcD() {}".to_string(),
        );

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        graph.add_node(node_d);

        // Create edges: A -> B, B -> C, C -> D
        let edge_ab = Edge::new(
            "test:a:1".to_string(),
            "funcB".to_string(),
            EdgeType::Calls,
            "funcB()".to_string(),
            PathBuf::from("test.go"),
            3,
        );

        let edge_bc = Edge::new(
            "test:b:10".to_string(),
            "funcC".to_string(),
            EdgeType::Calls,
            "funcC()".to_string(),
            PathBuf::from("test.go"),
            12,
        );

        let edge_cd = Edge::new(
            "test:c:20".to_string(),
            "funcD".to_string(),
            EdgeType::Calls,
            "funcD()".to_string(),
            PathBuf::from("test.go"),
            22,
        );

        graph.add_edge(edge_ab);
        graph.add_edge(edge_bc);
        graph.add_edge(edge_cd);

        graph
    }

    #[test]
    fn test_find_callers() {
        let graph = create_test_graph_with_calls();

        // funcB is called by funcA
        let callers = graph.find_callers("funcB");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].from, "test:a:1");

        // funcD is called by funcC
        let callers = graph.find_callers("funcD");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].from, "test:c:20");

        // funcA has no callers
        let callers = graph.find_callers("funcA");
        assert_eq!(callers.len(), 0);
    }

    #[test]
    fn test_trace_dependencies() {
        let graph = create_test_graph_with_calls();

        // Trace from funcA with depth 1 should find funcB
        let trace = graph.trace_dependencies("test:a:1", 1);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].to_name, "funcB");

        // Trace from funcA with depth 2 should find funcB and funcC
        let trace = graph.trace_dependencies("test:a:1", 2);
        assert_eq!(trace.len(), 2);

        // Trace from funcA with depth 3 should find all (B, C, D)
        let trace = graph.trace_dependencies("test:a:1", 3);
        assert_eq!(trace.len(), 3);
    }

    #[test]
    fn test_find_shortest_path() {
        let graph = create_test_graph_with_calls();

        // Find path from funcA to funcD
        let path = graph.find_shortest_path("test:a:1", "funcD", 10);
        assert!(path.is_some());

        let path = path.unwrap();
        // Path should be: B -> C -> D (edges traversed, not including start)
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "funcB");
        assert_eq!(path[1], "funcC");
        assert_eq!(path[2], "funcD");
    }

    #[test]
    fn test_find_shortest_path_no_path() {
        let graph = create_test_graph_with_calls();

        // No path from funcD to funcA (wrong direction)
        let path = graph.find_shortest_path("test:d:30", "funcA", 10);
        assert!(path.is_none());
    }

    #[test]
    fn test_find_shortest_path_depth_limit() {
        let graph = create_test_graph_with_calls();

        // Path exists but depth limit too small
        let path = graph.find_shortest_path("test:a:1", "funcD", 2);
        assert!(path.is_none());

        // With sufficient depth
        let path = graph.find_shortest_path("test:a:1", "funcD", 3);
        assert!(path.is_some());
    }

    #[test]
    fn test_find_paths_limited() {
        let graph = create_test_graph_with_calls();

        // Find 1 path from funcA to funcD
        let paths = graph.find_paths_limited("test:a:1", "funcD", 10, 1);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 4);
    }

    #[test]
    fn test_get_complexity() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        // Create a function that calls 3 others
        let node_main = Node::new(
            "test:main:1".to_string(),
            "main".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            10,
            "main".to_string(),
            "func main() {}".to_string(),
        );

        let node_a = Node::new(
            "test:a:15".to_string(),
            "funcA".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            15,
            20,
            "main".to_string(),
            "func funcA() {}".to_string(),
        );

        let node_b = Node::new(
            "test:b:25".to_string(),
            "funcB".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            25,
            30,
            "main".to_string(),
            "func funcB() {}".to_string(),
        );

        let node_c = Node::new(
            "test:c:35".to_string(),
            "funcC".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            35,
            40,
            "main".to_string(),
            "func funcC() {}".to_string(),
        );

        graph.add_node(node_main);
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // main calls A, B, C
        graph.add_edge(Edge::new(
            "test:main:1".to_string(),
            "funcA".to_string(),
            EdgeType::Calls,
            "funcA()".to_string(),
            PathBuf::from("test.go"),
            5,
        ));

        graph.add_edge(Edge::new(
            "test:main:1".to_string(),
            "funcB".to_string(),
            EdgeType::Calls,
            "funcB()".to_string(),
            PathBuf::from("test.go"),
            6,
        ));

        graph.add_edge(Edge::new(
            "test:main:1".to_string(),
            "funcC".to_string(),
            EdgeType::Calls,
            "funcC()".to_string(),
            PathBuf::from("test.go"),
            7,
        ));

        let complexity = graph.get_complexity("test:main:1");
        assert_eq!(complexity.fan_out, 3); // Calls 3 functions
        assert_eq!(complexity.fan_in, 0); // Called by none
    }

    #[test]
    fn test_find_hotspots() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        // Create a popular function called by many
        let popular = Node::new(
            "test:popular:1".to_string(),
            "popularFunc".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            5,
            "main".to_string(),
            "func popularFunc() {}".to_string(),
        );

        let caller1 = Node::new(
            "test:caller1:10".to_string(),
            "caller1".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            10,
            15,
            "main".to_string(),
            "func caller1() {}".to_string(),
        );

        let caller2 = Node::new(
            "test:caller2:20".to_string(),
            "caller2".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            20,
            25,
            "main".to_string(),
            "func caller2() {}".to_string(),
        );

        let caller3 = Node::new(
            "test:caller3:30".to_string(),
            "caller3".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            30,
            35,
            "main".to_string(),
            "func caller3() {}".to_string(),
        );

        graph.add_node(popular);
        graph.add_node(caller1);
        graph.add_node(caller2);
        graph.add_node(caller3);

        // All callers call popularFunc
        for i in 1..=3 {
            graph.add_edge(Edge::new(
                format!("test:caller{}:{}", i, i * 10),
                "popularFunc".to_string(),
                EdgeType::Calls,
                "popularFunc()".to_string(),
                PathBuf::from("test.go"),
                i * 10 + 2,
            ));
        }

        let hotspots = graph.find_hotspots(5);
        assert!(hotspots.len() > 0);
        assert_eq!(hotspots[0].name, "popularFunc");
        assert_eq!(hotspots[0].call_count, 3);
    }

    #[test]
    fn test_graph_merge() {
        let mut graph1 = CodeGraph::new("test".to_string(), "go".to_string());
        let mut graph2 = CodeGraph::new("test".to_string(), "go".to_string());

        // Add node to graph1
        graph1.add_node(Node::new(
            "test:a:1".to_string(),
            "funcA".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            5,
            "main".to_string(),
            "func funcA() {}".to_string(),
        ));

        // Add node to graph2
        graph2.add_node(Node::new(
            "test:b:10".to_string(),
            "funcB".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            10,
            15,
            "main".to_string(),
            "func funcB() {}".to_string(),
        ));

        // Merge
        graph1.merge(graph2);

        assert_eq!(graph1.nodes.len(), 2);
        assert!(graph1.get_node_by_id("test:a:1").is_some());
        assert!(graph1.get_node_by_id("test:b:10").is_some());
    }

    #[test]
    fn test_trace_handles_cycles() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        // Create circular dependency: A -> B -> C -> A
        let node_a = Node::new(
            "test:a:1".to_string(),
            "funcA".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            1,
            5,
            "main".to_string(),
            "func funcA() {}".to_string(),
        );

        let node_b = Node::new(
            "test:b:10".to_string(),
            "funcB".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            10,
            15,
            "main".to_string(),
            "func funcB() {}".to_string(),
        );

        let node_c = Node::new(
            "test:c:20".to_string(),
            "funcC".to_string(),
            NodeType::Function,
            PathBuf::from("test.go"),
            20,
            25,
            "main".to_string(),
            "func funcC() {}".to_string(),
        );

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // Create circular edges
        graph.add_edge(Edge::new(
            "test:a:1".to_string(),
            "funcB".to_string(),
            EdgeType::Calls,
            "funcB()".to_string(),
            PathBuf::from("test.go"),
            3,
        ));

        graph.add_edge(Edge::new(
            "test:b:10".to_string(),
            "funcC".to_string(),
            EdgeType::Calls,
            "funcC()".to_string(),
            PathBuf::from("test.go"),
            12,
        ));

        graph.add_edge(Edge::new(
            "test:c:20".to_string(),
            "funcA".to_string(),
            EdgeType::Calls,
            "funcA()".to_string(),
            PathBuf::from("test.go"),
            22,
        ));

        // Trace should handle cycles without infinite loop
        let trace = graph.trace_dependencies("test:a:1", 5);
        // Should find B and C, but not loop infinitely
        assert!(trace.len() >= 2);
        assert!(trace.len() <= 3); // Won't revisit A
    }

    #[test]
    fn test_outgoing_and_incoming_edges() {
        let graph = create_test_graph_with_calls();

        // funcA has 1 outgoing edge (to funcB)
        let outgoing = graph.get_outgoing_edges("test:a:1");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to, "funcB");

        // funcB has 1 incoming edge (from funcA - indexed by name) and 1 outgoing (to funcC)
        // Note: incoming edges are indexed by function name, not node ID
        let callers = graph.find_callers("funcB");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].from, "test:a:1");

        let outgoing = graph.get_outgoing_edges("test:b:10");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to, "funcC");
    }

    #[test]
    fn test_multiple_nodes_same_name() {
        let mut graph = CodeGraph::new("test".to_string(), "go".to_string());

        // Two functions with same name in different files
        let node1 = Node::new(
            "file1:helper:1".to_string(),
            "helper".to_string(),
            NodeType::Function,
            PathBuf::from("file1.go"),
            1,
            5,
            "main".to_string(),
            "func helper() {}".to_string(),
        );

        let node2 = Node::new(
            "file2:helper:1".to_string(),
            "helper".to_string(),
            NodeType::Function,
            PathBuf::from("file2.go"),
            1,
            5,
            "utils".to_string(),
            "func helper() {}".to_string(),
        );

        graph.add_node(node1);
        graph.add_node(node2);

        let helpers = graph.get_nodes_by_name("helper");
        assert_eq!(helpers.len(), 2);
    }
}
