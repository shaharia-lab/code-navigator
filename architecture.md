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

## Core Data Model

### Graph Structure
- **Nodes**: Functions, methods, classes (unique ID: file:name:line)
- **Edges**: Call relationships (caller → callee)
- **Indices**: Hash maps for O(1) lookups

### Index Types
```
node_by_id:    ID → node index           (exact match)
by_name:       Name → node indices       (functions with same name)
by_type:       Type → node indices       (all functions/methods/classes)
outgoing:      Node ID → edge indices    (downstream calls)
incoming:      Node name → edge indices  (upstream callers)
```

## Indexing Pipeline

### 1. Parallel File Discovery
```
Directory
  │
  ├─ Thread 1 ──▶ TypeScript files ──┐
  ├─ Thread 2 ──▶ Go files ──────────┤
  ├─ Thread 3 ──▶ Python files ───────┼──▶ Merge ──▶ Graph
  └─ Thread N ──▶ JavaScript files ──┘

Performance: ~50 files/second/thread
Library: jwalk (parallel directory walking)
```

### 2. Tree-sitter Parsing
- Language-agnostic syntax tree parsing
- Extract functions, methods, classes
- Identify call sites and relationships
- Build nodes (definitions) and edges (calls)

### 3. Incremental Merge
- Merge sub-graphs from parallel workers
- Update indices incrementally (no full rebuild)
- Pre-allocate capacity for better performance

### 4. Compression & Storage
```
JSON Serialize  ──▶  ~140 MB
     │
LZ4 Compress   ──▶  ~22 MB (6.4x smaller)
     │
Write to disk  ──▶  .bin file

Load time: ~1.08s (90K nodes)
```

## Navigation Commands

### Query
**Algorithm:** Hash-based index lookup
**Complexity:** O(1)

```
Filter by name  ──▶  by_name[name]      (exact match)
Filter by type  ──▶  by_type[type]      (function/method/class)
Multiple filters ──▶  Set intersection
```

**Performance:** <1ms for exact matches

### Trace
**Algorithm:** Depth-First Search
**Complexity:** O(E × D) where E=edges, D=depth

```
Start Node
    │
    ├─▶ Direct Call 1
    │      ├─▶ Nested Call 1.1
    │      └─▶ Nested Call 1.2
    │
    ├─▶ Direct Call 2
    │      └─▶ Nested Call 2.1
    └─▶ ...

DFS with visited tracking (prevents cycles)
Configurable depth limit
```

**Performance:** ~400ms for depth 1-3 (90K nodes)

### Callers
**Algorithm:** Reverse edge lookup
**Complexity:** O(1)

```
Function Name ──▶ incoming[name] ──▶ Edge indices ──▶ Callers
```

Direct hash map lookup, no iteration needed.

**Performance:** ~400ms even for 10K+ callers

### Path
**Two algorithms based on use case:**

#### Default: BFS (Shortest Path)
**Complexity:** O(V + E)

```
Start ──▶ Level 1 ──▶ Level 2 ──▶ Target
           │ │ │
Queue-based breadth-first traversal
First path found = shortest path
```

**Performance:** ~2s (90K nodes)
**Use case:** Most common - users want shortest path

#### --limit N: DFS (Multiple Paths)
**Complexity:** O(N^D) with early termination

```
Start
  ├─── Path 1 ───▶ Target  ✓
  ├─── Path 2 ───▶ Target  ✓
  └─── Path N ───▶ Target  ✓
       └── STOP (early termination)
```

**Optimization:** Use node indices (integers) during search, convert to names at end

**Performance:** ~8s for 10 paths (90K nodes)

### Analyze

#### Complexity Analysis
**Metric:** Fan-in (callers) + Fan-out (callees)
**Complexity:** O(N)

Uses pre-built indices for instant lookups.

#### Hotspots
**Metric:** Most frequently called functions
**Algorithm:** Count incoming edges per function
**Complexity:** O(E)

#### Coupling
**Metric:** Shared dependencies between functions
**Algorithm:** Dependency intersection
**Complexity:** O(N²) worst case

**Performance:** ~1.6s for full graph (90K nodes)

## Performance Profile

### Time Complexity

| Operation | Complexity | Time (90K nodes) |
|-----------|------------|------------------|
| Index | O(N × log N) | ~110s (5K files) |
| Load | O(N) | ~1.08s |
| Query | O(1) | <1ms |
| Trace | O(E × D) | ~400ms |
| Callers | O(1) | ~400ms |
| Path (BFS) | O(V + E) | ~2s |
| Path (DFS) | O(N^D) | ~8s (10 paths) |
| Analyze | O(N) to O(N²) | ~1.6s |

### Space Complexity

| Component | Size (90K nodes) |
|-----------|------------------|
| Nodes | ~5-10 MB |
| Edges | ~15-20 MB |
| Indices | ~50-60 MB |
| **Total Memory** | ~80-90 MB |
| **Disk (compressed)** | ~22 MB |

## Key Optimizations

### v0.3.0 - Query Speed (200x faster)
1. **Index-based lookups:** Hash maps for O(1) access
2. **Index caching:** Serialize indices to .idx file, skip rebuild on load
3. **LZ4 compression:** 3-4x faster decompression vs zstd

### v0.4.0 - Path Speed (15x faster)
1. **BFS for shortest path:** O(V+E) instead of O(N^D)
2. **Early termination:** Stop after finding N paths
3. **Index-based traversal:** Use integers instead of strings during search
4. **Smart defaults:** Shortest path by default (no flags needed)

### v0.2.0 - Indexing Speed (11.8% faster)
1. **Incremental merge:** Update indices during merge, no full rebuild
2. **Parallel processing:** jwalk + rayon for concurrent file parsing
3. **Batched processing:** Process files in chunks for better CPU utilization

## Storage Format

### Binary File (.bin)
```
┌─────────────────────────────┐
│  Magic: "CODENAV\x01"       │  8 bytes
├─────────────────────────────┤
│  Version: u32               │  4 bytes
├─────────────────────────────┤
│  LZ4 Compressed JSON Data   │  Variable
│    ├─ Nodes                 │
│    ├─ Edges                 │
│    └─ Metadata              │
└─────────────────────────────┘
```

### Index Cache (.idx)
```
┌─────────────────────────────┐
│  Version + Graph Hash       │  Validation
├─────────────────────────────┤
│  Zstd Compressed Indices    │
│    ├─ node_by_id            │
│    ├─ by_name               │
│    ├─ by_type               │
│    ├─ outgoing              │
│    └─ incoming              │
└─────────────────────────────┘
```

**Auto-managed:** Created on first load, validated by hash, can be safely deleted

## Algorithm Selection

### Command Decision Tree
```
Need exact function?        ──▶ query --name "func"
Need all of type?          ──▶ query --type function
Need downstream calls?     ──▶ trace --from "func" --depth N
Need upstream callers?     ──▶ callers "func"
Need shortest path?        ──▶ path --from A --to B
Need multiple paths?       ──▶ path --from A --to B --limit N
Need complexity analysis?  ──▶ analyze complexity
Need hotspots?            ──▶ analyze hotspots
```

### Performance Tradeoffs

| Approach | Speed | Completeness | Use Case |
|----------|-------|--------------|----------|
| Index lookup | ⚡⚡ Instant | Exact matches | Query, Callers |
| BFS | ⚡ Fast | Shortest path | Path (default) |
| DFS | 🐌 Slower | Multiple paths | Path --limit |
| Full scan | 🐌 Slow | All results | Analyze |

## Scalability

**Tested limits (VSCode codebase):**
- 5,275 files
- 90,022 nodes
- 200,000+ edges
- All operations <2 seconds

**Estimated capacity:**
- Up to 500K nodes: Performant
- Up to 10M edges: Acceptable
- Memory: ~1GB for very large graphs

## Design Principles

1. **Index everything:** Pre-compute for O(1) lookups
2. **Lazy loading:** Build indices only when needed
3. **Compression:** LZ4 for fast decompression
4. **Parallel parsing:** Utilize multiple cores
5. **Early termination:** Stop as soon as requirements met
6. **Smart defaults:** Optimize for common use case
