# Code Navigator

**Blazing-fast code graph generation for AI agents to navigate codebases efficiently.**

Code Navigator analyzes your codebase and builds a compressed graph of functions, calls, and dependencies. This allows AI agents to understand code architecture without expensive token usage from grepping or reading large portions of code.

## Why Code Navigator?

AI agents struggle with large codebases because:
- **Grepping is expensive**: Reading hundreds of files burns through tokens quickly
- **Context is limited**: LLMs can't hold entire codebases in memory
- **Navigation is slow**: Finding dependencies requires multiple read operations

Code Navigator solves this by:
- **Pre-computing relationships**: Generate once, query instantly
- **Compact representation**: 94% smaller than raw code (gzip compressed binary)
- **Lightning fast queries**: 32x faster than JSON parsing (sub-second loading)
- **Rich metadata**: Functions, calls, dependencies, complexity metrics

## Performance

Tested on 70,612 files (3.1M LOC):
- **Generation**: ~238 files/second
- **Query speed**: Sub-2 second loading for 50K+ nodes
- **Storage**: 94% smaller than JSON (139 MB → 8.7 MB)
- **Load time**: 32x faster (38s → 1.2s average)

## Installation

### Build from Source

Requires Rust 1.70+:

```bash
cargo build --release
sudo cp target/release/code-navigator /usr/local/bin/
```

### Download Binary

Download pre-built binaries from [releases](https://github.com/shaharia-lab/code-navigator/releases).

## Quick Start

```bash
# Generate code graph for your project
code-navigator generate /path/to/project --language typescript
# Output: code-navigator.bin

# Query functions
code-navigator query --name "authenticate*"

# Trace dependencies
code-navigator trace --from "processPayment" --depth 3

# Find callers
code-navigator callers "validateUser"

# Find call paths
code-navigator path --from "main" --to "saveDatabase"

# Analyze complexity
code-navigator analyze hotspots --threshold 10
```

## Supported Languages

- **Go** (.go): Functions, methods, packages
- **TypeScript** (.ts, .tsx): Functions, classes, async/await
- **JavaScript** (.js, .jsx): Functions, classes, modules
- **Python** (.py): Functions, classes, decorators

## Key Commands

### Generate Graph

```bash
code-navigator generate <DIRECTORY> [OPTIONS]

Options:
  -o, --output <FILE>      Output file (default: code-navigator.bin)
  -l, --language <LANG>    Language: go, typescript, javascript, python
  --incremental            Parse only changed files (faster updates)
  --exclude <PATTERN>      Exclude files matching pattern
```

### Query Nodes

```bash
code-navigator query [OPTIONS]

Options:
  --name <NAME>        Filter by name (supports wildcards: *auth*)
  --type <TYPE>        Filter by type: function, method, handler
  --file <PATH>        Filter by file path
  --count              Show count only
```

### Trace Dependencies

```bash
code-navigator trace --from <FUNCTION> [OPTIONS]

Options:
  -d, --depth <N>          Max depth (default: 1)
  -o, --output <FORMAT>    Output: tree, json, dot
  --show-lines             Show line numbers
```

### Find Callers

```bash
code-navigator callers <FUNCTION> [OPTIONS]

Options:
  -o, --output <FORMAT>    Output: tree, json, table
  --show-lines             Show line numbers
```

### Export to Other Formats

```bash
code-navigator export --format <FORMAT> -o <OUTPUT>

Formats: graphml, dot, csv
```

## Use Cases

### For AI Agents

Enable LLMs to:
- Navigate code without reading entire files
- Understand architectural patterns instantly
- Trace call chains for impact analysis
- Identify refactoring opportunities
- Generate accurate code modifications

### For Developers

- **Onboarding**: Quickly understand unfamiliar codebases
- **Refactoring**: Identify all affected code paths before changes
- **Code Review**: Detect complexity and coupling issues
- **Debugging**: Trace call chains to find root causes
- **Documentation**: Generate architectural diagrams automatically

### For CI/CD

- Track complexity metrics over time
- Detect architectural violations
- Monitor technical debt accumulation
- Validate dependency boundaries
- Generate release documentation

## Graph Format

Code Navigator uses gzip-compressed binary format:
- **94% smaller** than JSON (8.7 MB vs 139 MB for 70K files)
- **32x faster loading** (1.2s vs 38s average)
- **Backward compatible** (can read JSON/JSONL files)

## Example Output

```bash
$ code-navigator query --name "*auth*"

Name                  Type      File                          Line
------------------------------------------------------------------------
authenticateUser      Function  src/services/auth.ts          23
validateAuthToken     Function  src/middleware/auth.ts        45
checkAuthPermissions  Method    src/models/User.ts            89

→ 3 nodes found
```

```bash
$ code-navigator trace --from "authenticateUser" --depth 2

authenticateUser
  ├─ validateAuthToken
  │  └─ parseToken
  ├─ checkAuthPermissions
  └─ logAuthAttempt

→ 5 functions in dependency tree
```

## Architecture

```
code-navigator/
├── src/
│   ├── core/          # Graph data structures
│   ├── parser/        # Language parsers (tree-sitter)
│   ├── serializer/    # Format exporters
│   ├── cli.rs         # Command definitions
│   └── main.rs        # CLI handlers
└── tests/
    └── fixtures/      # Test files
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Benchmarks

See [FINAL_BENCHMARK_TABLE.md](FINAL_BENCHMARK_TABLE.md) for detailed performance data.

---

**Built for AI agents to navigate code at the speed of thought.**
