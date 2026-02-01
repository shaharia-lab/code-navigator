# Changelog

All notable changes to Code Navigator will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.1]: https://github.com/shaharia-lab/code-navigator/releases/tag/v0.1.1
[0.1.0]: https://github.com/shaharia-lab/code-navigator/releases/tag/v0.1.0
