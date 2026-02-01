# Changelog

All notable changes to Code Navigator will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-01

### 🚀 Major Performance Improvements

This release delivers **200x faster query performance** through intelligent use of hash-based indices and optimized data loading.

#### Query Performance (Primary Achievement) 🔥
- **Exact name queries**: Now <1ms (previously ~100-200ms) - **200x faster**
- **Type filter queries**: Now <1ms (previously ~100-200ms) - **200x faster**
- **Wildcard queries**: 8-14ms (optimized from linear scans)
- All query operations are now instant for exact matches

#### Load Performance
- **Overall load time**: 4-5% faster across all graph sizes
- **VSCode (90K nodes)**: 1.08s (previously 1.12s) - 4% improvement
- **Kubernetes (138K nodes)**: 1.55s (previously 1.63s) - 5% improvement

### Added
- **Index-based query execution**: Leverage existing hash indices for O(1) lookups instead of O(n) linear scans
- **Serialized index cache**: Automatically cache built indices to disk for faster subsequent loads
- **LZ4 compression**: Faster decompression compared to zstd (3-4x faster)
- **Timing instrumentation**: Use `--verbose` flag to see load vs query time breakdown
- **Optimal filter ordering**: Apply most selective filters first for better performance

### Changed
- **Default compression**: Switched from zstd to LZ4 for 3-4x faster decompression
- **Query execution strategy**: Now uses hash-based index lookups for exact matches
- **File format**: LZ4-compressed JSON (backward compatible with old formats)
- **Index caching**: Indices automatically saved/loaded from `.idx` files

### Performance Impact
All operations that query the graph now benefit from instant lookups:
- ✅ `query` - Find nodes by name/type → **<1ms**
- ✅ `callers` - Find function callers → **<1ms**
- ✅ `trace` - Trace dependencies → **Fast** (uses quick lookups)
- ✅ `path` - Find call paths → **Fast** (uses quick lookups)
- ✅ `analyze` - Code complexity analysis → **Fast**

### Technical Details
- Added `lz4_flex` dependency for faster compression
- Added `rmp-serde` dependency for future binary format support
- New `index_cache` module for transparent index persistence
- New `fast_compressed` module for LZ4 compression
- Graph hash validation ensures cache correctness
- Backward compatible: Can still load old zstd/JSON formats

### Trade-offs
- **File size increase**: LZ4 produces 50-60% larger files than zstd
  - VSCode: 22 MB (was 14 MB)
  - Kubernetes: 27 MB (was 17 MB)
  - Acceptable trade-off for 200x query speed improvement

### Notes
- Indexing performance unchanged - parsing speed unaffected
- No breaking changes - fully backward compatible
- Cache files (`.idx`) automatically managed, can be safely deleted

## [0.2.0] - 2026-02-01

### Performance 🚀

**Major performance improvements across the board - 11.8% faster overall!**

- **27x faster serialization**: Saving time reduced from 15.8s to 0.55s (-96.5%)
- **11.8% faster overall**: Total indexing time improved from 120s to 106s
- **12% higher throughput**: Processing speed increased from 44.5 to 49.8 files/sec
- **48% more consistent**: Standard deviation reduced from ±5.79s to ±2.98s

#### Benchmark Results

**TypeScript (VSCode - 5,275 files, 2M LOC)**:
- Average: 109.52s ±1.57s
- Throughput: 48.2 files/sec | 18,249 LOC/sec
- Memory: 83.5 MB peak

**Golang (Kubernetes - 13,741 files, 4.9M LOC)**:
- Average: 427.41s ±2.56s
- Throughput: 32.2 files/sec | 11,457 LOC/sec
- Memory: 129.8 MB peak

### Changed

- **Phase 1**: Incremental index updates during merge instead of full rebuild
  - Eliminates expensive O(N+E) index reconstruction
  - ~5.4% performance improvement
- **Phase 2**: JSON + Zstd compression instead of JSON + Gzip
  - Zstd is 2-3x faster than Gzip at similar compression ratios
  - ~4.9% performance improvement
  - 27x faster serialization
- **Phase 3**: Parallel file discovery and batched processing
  - Replaced `walkdir` with `jwalk` for parallel directory walking
  - Process files in chunks of 100 for better CPU utilization
  - ~2.0% performance improvement

### Added

- `new_with_capacity()` constructor for pre-allocated HashMaps
- `indices_dirty` flag for lazy index rebuilding (future optimization)
- `ensure_indices()` method for on-demand index updates
- Comprehensive benchmark mode with detailed metrics
- Dependencies: `zstd` (v0.13), `jwalk` (v0.8)

### Technical Details

**Phase 1: Incremental Merge Optimization**
- Merge now updates indices incrementally instead of rebuilding
- Pre-allocates capacity for better memory management
- Removes unnecessary `build_indexes()` calls

**Phase 2: Storage Format Optimization**
- JSON + Zstd provides excellent compatibility and performance
- Maintains full serde attribute support
- Simplified save/load implementation

**Phase 3: Parallel Processing**
- Batched processing reduces merge overhead
- Parallel file discovery improves startup time
- Applied to TypeScript, Go, and Python parsers

### Scaling Projections

| Codebase Size | Before | After | Time Saved |
|---------------|--------|-------|------------|
| 5K files | 2 min | 1.8 min | 14 seconds |
| 50K files | 20 min | 17.5 min | 2.5 minutes |
| 500K files | 3.3 hrs | 2.9 hrs | 24 minutes |
| 1M files | 6.7 hrs | 5.9 hrs | 48 minutes |

### Notes

- All optimizations are backward compatible
- No breaking changes to API or CLI
- Output file format updated (JSON+Zstd instead of JSON+Gzip)
- Old `.bin` files can still be loaded

## [0.1.1] - 2026-02-01

### Changed
- **BREAKING**: Renamed binary from `code-navigator` to `codenav` for easier terminal usage
- **BREAKING**: Renamed `generate` command to `index` for better semantic clarity
  - `codenav generate` → `codenav index`
  - More intuitive: "index a codebase" vs "generate a graph"
  - Industry-standard terminology (like search engines, databases)
- Default output filename changed from `code-navigator.bin` to `codenav.bin`
- Updated CLI output messages:
  - "Generating code graph..." → "Indexing codebase..."
  - "Generated graph with X nodes" → "Indexed X nodes"

### Improved
- Comprehensive README with clear vision and goals
- Platform-specific installation guides (Linux, macOS with Intel/Apple Silicon, Windows)
- Collapsible sections for better README scannability
- Navigation philosophy: Emphasizes token efficiency for AI agents
- Clear explanation: "Code navigation is not just a search problem"
- Added comprehensive FAQ section (5 questions)
- Updated all examples to use new `codenav index` command

### Fixed
- Release workflow asset names updated to match new binary name
- CI workflow artifact names updated to match new binary name

## [0.1.0] - 2026-02-01

### Added
- Initial release of Code Navigator
- Multi-language support: Go, TypeScript, JavaScript, Python
- Blazing-fast code graph generation (~238 files/second)
- Compressed binary format (94% smaller than JSON, 32x faster loading)
- Query command for searching functions by name, type, file, package
- Trace command for dependency analysis (downstream calls)
- Callers command for reverse dependency lookup (upstream calls)
- Path command for finding call paths between functions
- Analyze command with hotspots, complexity, coupling, and circular dependency detection
- Export command for GraphML, DOT, and CSV formats
- Extract command for focused subgraph generation
- Diff command for comparing graphs
- Incremental update mode for faster regeneration
- Tree-sitter based parsing for accurate code analysis
- GitHub Actions CI/CD workflows for testing and releases
- Pre-built binaries for Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), Windows (x86_64)

### Performance
- Generation: ~238 files/second (tested on 70,612 files, 3.1M LOC)
- Query speed: Sub-2 second loading for 50K+ nodes
- Storage: 94% smaller than JSON (139 MB → 8.7 MB)
- Load time: 32x faster than JSON (38s → 1.2s average)

[0.2.0]: https://github.com/shaharia-lab/code-navigator/releases/tag/v0.2.0
[0.1.1]: https://github.com/shaharia-lab/code-navigator/releases/tag/v0.1.1
[0.1.0]: https://github.com/shaharia-lab/code-navigator/releases/tag/v0.1.0
