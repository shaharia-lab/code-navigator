# Code Navigator - Technical Architecture

## System Overview

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Source    │────▶│   Indexing   │────▶│    Graph    │
│   Code      │     │   (Parse)    │     │   Storage   │
└─────────────┘     └──────────────┘     └─────────────┘
                                                 │
                                                 ▼
                          ┌──────────────────────────────────┐
                          │   Navigation Commands            │
                          │  ┌────────┬────────┬──────────┐  │
                          │  │ Query  │ Trace  │ Callers  │  │
                          │  ├────────┼────────┼──────────┤  │
                          │  │ Path   │ Analyze│ Export   │  │
                          │  └────────┴────────┴──────────┘  │
                          └──────────────────────────────────┘
```

## Core Data Structures

### CodeGraph
```rust
struct CodeGraph {
    nodes: Vec<Node>,           // All functions/methods/classes
    edges: Vec<Edge>,           // Call relationships

    // Hash indices (O(1) lookups)
    node_by_id: HashMap<String, usize>,           // ID → node index
    by_name: HashMap<String, Vec<usize>>,         // Name → node indices
    by_type: HashMap<NodeType, Vec<usize>>,       // Type → node indices
    outgoing: HashMap<String, Vec<usize>>,        // Node ID → outgoing edges
    incoming: HashMap<String, Vec<usize>>,        // Node name → incoming edges
}
```

### Node (Function/Method/Class)
```rust
struct Node {
    id: String,              // Unique: file:name:line
    name: String,            // Function name
    node_type: NodeType,     // Function, Method, Class, etc.
    file_path: PathBuf,      // Source file location
    line: usize,             // Start line
    signature: String,       // Full signature
}
```

### Edge (Call Relationship)
```rust
struct Edge {
    from: String,            // Caller node ID
    to: String,              // Callee function name
    edge_type: EdgeType,     // Direct, Virtual, etc.
    call_site_line: usize,   // Where the call happens
}
```

## Indexing Phase

### 1. Parallel File Discovery
```
Directory Tree
     │
     ├─ Thread 1 ──▶ *.ts files ──▶ TypeScript Parser ──┐
     ├─ Thread 2 ──▶ *.go files ──▶ Go Parser ──────────┤
     ├─ Thread 3 ──▶ *.py files ──▶ Python Parser ───────┼──▶ Merge ──▶ Graph
     └─ Thread N ──▶ *.js files ──▶ JavaScript Parser ──┘

Performance: ~50 files/second per thread
Parallelism: jwalk for directory walking
```

### 2. Tree-sitter Parsing
```
Source Code
     │
     ▼
┌──────────────────┐
│  Tree-sitter     │  Syntax tree parsing
│  Parser          │  Language-agnostic
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  AST Traversal   │  Extract functions/calls
│                  │  Build nodes & edges
└────────┬─────────┘
         │
         ▼
    Sub-Graph
```

### 3. Graph Merge (Incremental)
```rust
// O(N) merge with incremental index updates
for node in other_graph.nodes {
    idx = self.nodes.len();
    self.nodes.push(node);
    self.node_by_id.insert(node.id, idx);      // Update index incrementally
    self.by_name[node.name].push(idx);         // No full rebuild needed
}
```

### 4. Serialization & Compression
```
Graph (in-memory)
     │
     ▼
JSON Serialization      ────▶ ~140 MB
     │
     ▼
LZ4 Compression         ────▶ ~22 MB (6.4x smaller)
     │
     ▼
Disk Storage (.bin)
```

**Load Performance:**
- LZ4 decompress: ~300ms
- JSON deserialize: ~600ms
- Index load/build: ~180ms
- **Total: ~1.08s** (for 90K nodes)

## Query Operations

### Query Command
**Algorithm:** Hash-based index lookup
**Complexity:** O(1)

```rust
// Exact name match
nodes = graph.by_name.get(name);           // O(1) hash lookup

// Type filter
nodes = graph.by_type.get(type);           // O(1) hash lookup

// Multiple filters: set intersection
result = name_set ∩ type_set ∩ file_set;   // O(min(|sets|))
```

**Performance:** <1ms for exact matches

### Trace Command
**Algorithm:** DFS with depth limit
**Complexity:** O(E × D) where E=edges, D=depth

```
Start Node
    │
    ├─▶ Dependency 1
    │      ├─▶ Sub-dep 1.1
    │      └─▶ Sub-dep 1.2
    │
    ├─▶ Dependency 2
    │      └─▶ Sub-dep 2.1
    │             └─▶ Sub-dep 2.1.1
    └─▶ ...

DFS traversal with visited set to avoid cycles
```

```rust
fn trace_recursive(node_id, depth, max_depth, visited, results) {
    if depth >= max_depth || visited.contains(node_id) {
        return;  // Stop at depth limit or cycles
    }
    visited.insert(node_id);

    for edge in graph.get_outgoing_edges(node_id) {
        results.push(edge);
        trace_recursive(edge.to, depth + 1, max_depth, visited, results);
    }
}
```

**Performance:** ~400ms for depth 1-3 (90K nodes)

### Callers Command
**Algorithm:** Reverse edge lookup
**Complexity:** O(1)

```
Function Name
     │
     ▼
incoming[name]  ────▶  [edge_idx1, edge_idx2, ...]
     │
     ▼
[Edge1, Edge2, Edge3, ...]
```

```rust
// Direct index lookup - no iteration needed
callers = graph.incoming.get(function_name);  // O(1)
edges = callers.map(|indices|
    indices.iter().map(|&i| &graph.edges[i])
);
```

**Performance:** ~400ms even for 10K+ callers

### Path Command
**Algorithm:** BFS (shortest path) or DFS (multiple paths)
**Complexity:** O(V + E) for BFS, O(N^D) for DFS

#### BFS (Default - Shortest Path)
```
Start ──▶ Level 1 ──▶ Level 2 ──▶ ... ──▶ Target
  │         │ │ │       │ │ │
  └─────────┴─┴─┴───────┴─┴─┴─── Queue-based traversal
                                  First path found = shortest
```

```rust
fn find_shortest_path(from, to, max_depth) {
    queue = [from];
    parent = HashMap::new();

    while let Some(current) = queue.pop_front() {
        for edge in graph.get_outgoing_edges(current) {
            if edge.to == to {
                return reconstruct_path(parent, from, current, to);  // Found!
            }
            if !visited.contains(edge.to) {
                queue.push_back(edge.to);
                parent[edge.to] = current;
            }
        }
    }
}
```

**Performance:** ~2s for 90K nodes (was 30+ sec with old DFS)

#### DFS (Multiple Paths with --limit N)
```
Start
  ├─── Path 1 ───▶ Target  ✓
  ├─── Path 2 ───▶ Target  ✓
  ├─── Path 3 ─X  (dead end)
  └─── Path 4 ───▶ Target  ✓
       │
       └── STOP after N paths found (early termination)
```

**Optimization:** Index-based traversal using `Vec<usize>` instead of `Vec<String>`

```rust
// Phase 3 optimization: Use indices during search
fn find_paths_by_index(from_idx: usize, to_name, max_depth, max_paths) {
    path: Vec<usize> = vec![from_idx];        // Indices, not strings
    visited: HashSet<usize> = HashSet::new(); // Integer comparisons

    // DFS with early termination
    dfs(from_idx, to_name, &mut path, &mut visited, max_paths);

    // Convert to names only at the end
    paths.map(|p| convert_indices_to_names(p))
}
```

**Performance:** ~8s for 10 paths (was 31s before optimization)

### Analyze Command

#### Complexity Analysis
**Algorithm:** Fan-in/Fan-out calculation
**Complexity:** O(N) where N=nodes

```rust
for node in graph.nodes {
    fan_out = graph.outgoing[node.id].len();     // O(1)
    fan_in = graph.incoming[node.name].len();    // O(1)
    complexity = fan_in + fan_out + 1;
}
```

#### Hotspots (Most Called Functions)
**Algorithm:** Aggregate incoming edge counts
**Complexity:** O(E) where E=edges

```rust
hotspots = HashMap::new();
for edge in graph.edges {
    hotspots[edge.to] += 1;  // Count calls to each function
}
hotspots.sort_by_value().take(N);
```

#### Coupling Analysis
**Algorithm:** Shared dependencies detection
**Complexity:** O(N²) in worst case

```rust
for node1 in graph.nodes {
    deps1 = get_dependencies(node1);
    for node2 in graph.nodes {
        deps2 = get_dependencies(node2);
        coupling = deps1.intersection(deps2).count();
    }
}
```

**Performance:** ~1.6s for 90K nodes

## Performance Characteristics

### Time Complexity Summary

| Operation | Algorithm | Complexity | Actual Time (90K nodes) |
|-----------|-----------|------------|-------------------------|
| **Index** | Tree-sitter + Merge | O(N × log N) | ~110s (5K files) |
| **Load** | LZ4 + JSON | O(N) | ~1.08s |
| **Query** | Hash lookup | O(1) | <1ms |
| **Trace** | DFS | O(E × D) | ~400ms |
| **Callers** | Index lookup | O(1) | ~400ms |
| **Path (BFS)** | BFS | O(V + E) | ~2s |
| **Path (DFS)** | DFS + Early stop | O(N^D) | ~8s (10 paths) |
| **Analyze** | Linear scan | O(N) to O(N²) | ~1.6s |

### Space Complexity

| Component | Size (90K nodes) | Notes |
|-----------|------------------|-------|
| Nodes | ~5-10 MB | Vec<Node> in memory |
| Edges | ~15-20 MB | Vec<Edge> in memory |
| Indices | ~50-60 MB | HashMap structures |
| **Total Memory** | ~80-90 MB | Peak RSS |
| **Disk (compressed)** | ~22 MB | LZ4 + JSON |

## Key Optimizations

### v0.3.0 - Query Optimization (200x faster)
- **Index-based lookups:** O(1) hash map access
- **Serialized index cache:** Skip rebuild on load
- **LZ4 compression:** 3-4x faster decompression

### v0.4.0 - Path Optimization (15x faster)
- **BFS for shortest path:** O(V+E) instead of O(N^D)
- **Early termination:** Stop after N paths found
- **Index-based traversal:** Use `usize` instead of `String`
- **Smart defaults:** Shortest path without flags

### Incremental Merge (v0.2.0)
- **Parallel parsing:** jwalk + rayon for concurrency
- **Incremental updates:** Update indices during merge
- **No rebuilds:** Avoid O(N) index reconstruction

## Storage Format

### Binary Format (.bin)
```
┌──────────────────────────────┐
│  Magic Bytes: "CODENAV\x01"  │  8 bytes
├──────────────────────────────┤
│  Format Version: u32         │  4 bytes
├──────────────────────────────┤
│  LZ4 Compressed Data         │  Variable
│    ├─ JSON Serialized Graph  │
│    └─ All nodes & edges      │
└──────────────────────────────┘
```

### Index Cache (.idx)
```
┌──────────────────────────────┐
│  Version String              │
├──────────────────────────────┤
│  Graph Hash (validation)     │
├──────────────────────────────┤
│  Node/Edge Counts            │
├──────────────────────────────┤
│  Zstd Compressed Indices     │
│    ├─ node_by_id             │
│    ├─ by_name                │
│    ├─ by_type                │
│    ├─ outgoing               │
│    └─ incoming               │
└──────────────────────────────┘
```

**Auto-managed:** Created on first load, validated by hash

## Algorithm Selection Guide

### When to Use Each Command

```
Need shortest path?        ──▶ path (default, BFS)
Need multiple paths?       ──▶ path --limit N (DFS)
Need downstream calls?     ──▶ trace --depth N (DFS)
Need upstream callers?     ──▶ callers (index lookup)
Need complexity metrics?   ──▶ analyze complexity
Need popular functions?    ──▶ analyze hotspots
```

### Performance Tradeoffs

| Feature | Speed | Completeness | Use Case |
|---------|-------|--------------|----------|
| BFS (path) | ⚡ Fast | Shortest only | Default navigation |
| DFS (path) | 🐌 Slower | Multiple paths | Exploration |
| Index lookup | ⚡⚡ Instant | Exact matches | Direct queries |
| Full scan | 🐌 Slow | Complete | Analysis tasks |

## Scalability Limits

**Tested on VSCode codebase:**
- 5,275 TypeScript files
- 90,022 nodes (functions/methods)
- 200,000+ edges (calls)
- **All operations: <2 seconds**

**Estimated limits:**
- Up to 500K nodes: Still performant
- Up to 10M edges: Acceptable
- Memory limit: ~1GB for very large graphs

## Backward Compatibility

**Supports multiple formats:**
- LZ4 + JSON (current, default)
- Zstd + JSON (v0.3.0)
- Plain JSON (v0.1.0)
- Gzip + JSON (v0.1.0)

**Auto-detection:** Magic bytes identify format
**Fallback:** Graceful degradation to older formats
