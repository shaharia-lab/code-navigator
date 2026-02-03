# Code Navigator

> 🚀 **Blazing-fast code indexing for AI agents to navigate large codebases efficiently.**

Code Navigator solves a critical problem for terminal-based AI agents: **navigating large codebases without burning through tokens**. Instead of expensive grepping operations that read hundreds of files, Code Navigator pre-computes a compressed graph of your entire codebase, enabling instant queries and navigation.

## 🎯 Why Code Navigator?

### The Token Problem

When AI agents work with large codebases, **every grep is a tax**. Traditional approaches require:
- Reading multiple files to find a function definition
- Scanning dozens of files to trace dependencies
- Opening entire files just to understand call relationships

Each operation consumes tokens, slows down the agent, and increases costs. For a 50,000-line codebase, a simple "find all callers of function X" could consume thousands of tokens.

### Code Navigation is Not Just Search

**We believe code retrieval is fundamentally a navigation problem, not just a search problem.**

- **Search** finds text matches: "where does string 'foo' appear?"
- **Navigation** understands relationships: "what calls this function?", "how do these modules depend on each other?"

Traditional tools like `grep`, `rg`, or `ast-grep` are excellent for search, but they don't understand the **graph structure** of your code. Code Navigator pre-computes this graph, allowing AI agents to navigate relationships instantly without re-parsing code.

### How Code Navigator Solves This

- 🗜️ **Pre-computed graph**: Index once, query thousands of times
- ⚡ **Token efficient**: Query the graph instead of reading files
- 🎯 **Relationship-aware**: Navigate calls, dependencies, and complexity
- 📦 **Compact storage**: 94% smaller than raw code (gzip compressed)
- 🚄 **Lightning fast**: Sub-second queries on 50K+ node graphs

**Example savings**: Finding all callers of a function in a 50K LOC project:
- Traditional grep approach: ~2,000 tokens (reading multiple files)
- Code Navigator: ~50 tokens (single graph query)

## 📦 Installation

<details open>
<summary><b>Option 1: Homebrew (macOS & Linux)</b></summary>
<br>

The easiest way to install Code Navigator:

```bash
# Add the tap (one-time setup)
brew tap shaharia-lab/tap

# Install codenav
brew install codenav

# Verify installation
codenav --version
```

To upgrade to the latest version:
```bash
brew update && brew upgrade codenav
```

</details>

<details>
<summary><b>Option 2: Download Pre-built Binary</b></summary>
<br>

#### 🐧 Linux / Ubuntu

1. **Download the binary**:
   ```bash
   wget https://github.com/shaharia-lab/code-navigator/releases/latest/download/codenav-linux-x86_64.tar.gz
   ```

2. **Extract the archive**:
   ```bash
   tar -xzf codenav-linux-x86_64.tar.gz
   ```

3. **Move to system path**:
   ```bash
   sudo mv codenav /usr/local/bin/
   ```

4. **Verify installation**:
   ```bash
   codenav --version
   ```

#### 🍎 macOS

1. **Download the binary** (Intel or Apple Silicon):
   ```bash
   # For Intel Macs
   curl -LO https://github.com/shaharia-lab/code-navigator/releases/latest/download/codenav-macos-x86_64.tar.gz
   tar -xzf codenav-macos-x86_64.tar.gz

   # For Apple Silicon (M1/M2/M3)
   curl -LO https://github.com/shaharia-lab/code-navigator/releases/latest/download/codenav-macos-aarch64.tar.gz
   tar -xzf codenav-macos-aarch64.tar.gz
   ```

2. **Move to system path**:
   ```bash
   sudo mv codenav /usr/local/bin/
   ```

3. **Remove quarantine attribute** (macOS security):
   ```bash
   xattr -d com.apple.quarantine /usr/local/bin/codenav
   ```

4. **Verify installation**:
   ```bash
   codenav --version
   ```

#### 🪟 Windows

1. **Download the binary**:
   - Go to [releases page](https://github.com/shaharia-lab/code-navigator/releases/latest)
   - Download `codenav-windows-x86_64.exe.zip`

2. **Extract the archive**:
   - Right-click the downloaded ZIP file
   - Select "Extract All..."
   - Choose a destination folder

3. **Add to PATH**:
   - Press `Win + X`, select "System"
   - Click "Advanced system settings"
   - Click "Environment Variables"
   - Under "User variables", select "Path" and click "Edit"
   - Click "New" and add the folder containing `codenav.exe`
   - Click "OK" to save

4. **Verify installation**:
   ```powershell
   codenav --version
   ```

</details>

<details>
<summary><b>Option 3: Build from Source</b></summary>
<br>

Requires Rust 1.70 or later:

```bash
# Clone the repository
git clone https://github.com/shaharia-lab/code-navigator.git
cd code-navigator

# Build release binary
cargo build --release

# Install (Linux/macOS)
sudo cp target/release/codenav /usr/local/bin/

# Or install (Windows, run as Administrator)
copy target\release\codenav.exe C:\Windows\System32\
```

</details>

## 🚀 Quick Start

```bash
# Index your project to build a code graph
codenav index /path/to/project --language typescript
# Output: codenav.bin

# Query functions by name (supports wildcards)
codenav query --name "authenticate*"

# Find who calls a function (reverse dependencies)
codenav callers "processPayment"

# Trace function dependencies (downstream calls)
codenav trace --from "validateUser" --depth 3

# Find call paths between two functions
codenav path --from "main" --to "saveDatabase"

# Analyze code complexity and hotspots
codenav analyze hotspots --threshold 10
```
## Claude Code Skills

For Claude Code users, you can install [Code Navigator plugins](https://github.com/shaharia-lab/claude-power-user/tree/main/plugins/code-navigator)
to get the `/codenav-navigation` skill. So you can use `/codenav-navigation <funcName or your question>` directly in your Claude Code.

You can also use [this skill](https://github.com/shaharia-lab/claude-power-user/blob/main/plugins/code-navigator/skills/codenav-navigation/SKILL.md) as a prompt in any other AI tools.

## 🔧 Supported Languages

| Language | Extensions | Features |
|----------|-----------|----------|
| **Go** | `.go` | Functions, methods, packages, interfaces |
| **TypeScript** | `.ts`, `.tsx` | Functions, classes, async/await, React components |
| **JavaScript** | `.js`, `.jsx` | Functions, classes, modules, React components |
| **Python** | `.py` | Functions, classes, decorators, async/await |

More languages coming soon! See [CONTRIBUTING.md](CONTRIBUTING.md) to add language support.

## 📖 Usage

<details>
<summary><b>Index Codebase</b></summary>

Index a codebase to build a navigable code graph:

```bash
codenav index <DIRECTORY> [OPTIONS]

Options:
  -o, --output <FILE>      Output file (default: codenav.bin)
  -l, --language <LANG>    Language: go, typescript, javascript, python
  --incremental            Parse only changed files (faster updates)
  --exclude <PATTERN>      Exclude files matching pattern (can specify multiple times)
  --include-tests          Include test files in the graph
  --force                  Force full reindexing even with --incremental
  --benchmark              Enable comprehensive performance metrics
  --benchmark-json <FILE>  Export benchmark results to JSON file (requires --benchmark)

Examples:
  # Index a TypeScript project
  codenav index ./my-app --language typescript

  # Index with exclusions
  codenav index ./my-app -l typescript --exclude "*.test.ts" --exclude "node_modules/*"

  # Incremental update (only index changed files)
  codenav index ./my-app -l typescript --incremental

  # Index with performance benchmarking
  codenav index ./my-app -l typescript --benchmark

  # Export benchmark metrics to JSON for analysis
  codenav index ./my-app -l typescript --benchmark --benchmark-json metrics.json
```

</details>

<details>
<summary><b>Query Nodes</b></summary>

Search for functions, classes, or methods:

```bash
codenav query [OPTIONS]

Options:
  --name <NAME>        Filter by name (supports wildcards: *auth*)
  --type <TYPE>        Filter by type: function, method, handler, class
  --file <PATH>        Filter by file path (supports wildcards)
  --package <NAME>     Filter by package/module name
  --count              Show count only (no details)

Examples:
  # Find all authentication-related functions
  codenav query --name "*auth*"

  # Find all handler functions
  codenav query --type handler

  # Find functions in specific file
  codenav query --file "src/services/*.ts"

  # Just get the count
  codenav query --name "test*" --count
```

</details>

<details>
<summary><b>Trace Dependencies</b></summary>

Find all functions called by a given function (downstream dependencies):

```bash
codenav trace --from <FUNCTION> [OPTIONS]

Options:
  -d, --depth <N>          Max depth to traverse (default: 1)
  -o, --output <FORMAT>    Output format: tree, json, dot
  --show-lines             Show line numbers in output
  --graph <FILE>           Use specific graph file (default: codenav.bin)

Examples:
  # Show immediate dependencies
  codenav trace --from "processPayment"

  # Show deep dependency tree
  codenav trace --from "processPayment" --depth 5

  # Export as DOT graph for visualization
  codenav trace --from "processPayment" -o dot > deps.dot
```

</details>

<details>
<summary><b>Find Callers</b></summary>

Find all functions that call a given function (reverse dependencies):

```bash
codenav callers <FUNCTION> [OPTIONS]

Options:
  -o, --output <FORMAT>    Output format: tree, json, table
  --show-lines             Show line numbers
  --graph <FILE>           Use specific graph file

Examples:
  # Find who calls this function
  codenav callers "validateUser"

  # Output as table
  codenav callers "validateUser" -o table

  # Show with line numbers
  codenav callers "validateUser" --show-lines
```

</details>

<details>
<summary><b>Find Call Paths</b></summary>

Find all possible paths between two functions:

```bash
codenav path --from <FUNCTION> --to <FUNCTION> [OPTIONS]

Options:
  --max-depth <N>     Maximum path length (default: 10)
  --graph <FILE>      Use specific graph file

Examples:
  # Find how main() reaches saveToDatabase()
  codenav path --from "main" --to "saveToDatabase"

  # Limit path length
  codenav path --from "handleRequest" --to "queryDB" --max-depth 5
```

</details>

<details>
<summary><b>Analyze Code Complexity</b></summary>

Identify complexity hotspots and coupling issues:

```bash
codenav analyze <SUBCOMMAND> [OPTIONS]

Subcommands:
  hotspots     Find high-complexity functions
  coupling     Find highly coupled modules
  circular     Detect circular dependencies

Examples:
  # Find functions with complexity > 10
  codenav analyze hotspots --threshold 10

  # Find highly coupled modules
  codenav analyze coupling --min-connections 15

  # Detect circular dependencies
  codenav analyze circular
```

</details>

<details>
<summary><b>Compare Graphs (Diff)</b></summary>

Compare two code graphs to see what changed:

```bash
codenav diff <OLD_GRAPH> <NEW_GRAPH> [OPTIONS]

Options:
  --show-added         Show added nodes
  --show-removed       Show removed nodes
  --show-changed       Show modified nodes
  --complexity-threshold <N>  Highlight complexity changes > N

Examples:
  # Compare before and after refactoring
  codenav diff old-graph.bin new-graph.bin

  # Show only added functions
  codenav diff old.bin new.bin --show-added

  # Highlight significant complexity changes
  codenav diff old.bin new.bin --complexity-threshold 5
```

</details>

<details>
<summary><b>Export Graph</b></summary>

Export the graph to other formats for visualization or analysis:

```bash
codenav export --format <FORMAT> -o <OUTPUT> [OPTIONS]

Formats:
  graphml    GraphML (for Gephi, yEd)
  dot        DOT/Graphviz (for visualization)
  csv        CSV (for spreadsheet analysis)

Examples:
  # Export to GraphML for visualization in Gephi
  codenav export --format graphml -o graph.graphml

  # Export to DOT and render with Graphviz
  codenav export --format dot -o graph.dot
  dot -Tpng graph.dot -o graph.png
```

</details>

## 💡 Example Output

<details>
<summary><b>See example outputs</b></summary>
<br>

**Querying functions:**
```bash
$ codenav query --name "*auth*"

Name                  Type      File                          Line
------------------------------------------------------------------------
authenticateUser      Function  src/services/auth.ts          23
validateAuthToken     Function  src/middleware/auth.ts        45
checkAuthPermissions  Method    src/models/User.ts            89

→ 3 nodes found
```

**Tracing dependencies:**
```bash
$ codenav trace --from "authenticateUser" --depth 2

authenticateUser
  ├─ validateAuthToken
  │  └─ parseJWT
  ├─ checkAuthPermissions
  │  └─ queryUserRoles
  └─ logAuthAttempt

→ 6 functions in dependency tree
```

**Finding callers:**
```bash
$ codenav callers "validateAuthToken"

validateAuthToken is called by:
  ├─ authenticateUser (src/services/auth.ts:23)
  ├─ refreshToken (src/services/auth.ts:67)
  └─ checkSession (src/middleware/session.ts:34)

→ 3 callers found
```

</details>

## 🎯 Use Cases

<details>
<summary><b>For AI Agents 🤖</b></summary>

Enable LLMs to navigate code efficiently:
- **Token optimization**: Query the graph instead of reading files
- **Instant relationship lookup**: "What calls this?", "What does this call?"
- **Impact analysis**: Understand the ripple effects of changes
- **Architectural understanding**: Grasp module boundaries and dependencies
- **Refactoring guidance**: Identify safe refactoring opportunities

**Example workflow:**
```
Agent: "I need to modify function authenticateUser"
1. code-navigator callers "authenticateUser"  # Find impacted code
2. code-navigator trace --from "authenticateUser"  # Understand dependencies
3. Make informed changes with full context
```

</details>

<details>
<summary><b>For Developers 👨‍💻</b></summary>

- **Onboarding**: Quickly understand unfamiliar codebases
- **Refactoring**: Identify all affected code paths before changes
- **Code Review**: Detect complexity and coupling issues
- **Debugging**: Trace call chains to find root causes
- **Documentation**: Generate architectural diagrams automatically

</details>

<details>
<summary><b>For CI/CD Pipelines 🔄</b></summary>

- Track complexity metrics over time
- Detect architectural violations
- Monitor technical debt accumulation
- Validate dependency boundaries
- Generate release documentation

</details>

## ❓ FAQ

<details>
<summary><b>What format does Code Navigator use for storage?</b></summary>

Code Navigator uses a **gzip-compressed binary format** (`.bin` files) by default. This provides:
- **94% smaller** file size compared to JSON (8.7 MB vs 139 MB for 70K files)
- **32x faster loading** (1.2s vs 38s average)
- **Backward compatibility**: Can still read JSON/JSONL files from other tools

The binary format is just gzip-compressed JSON, so you can decompress it if needed:
```bash
gunzip -c codenav.bin | jq .
```

</details>

<details>
<summary><b>How is this different from grep, ripgrep, or ast-grep?</b></summary>

Traditional tools are excellent for **text search**, but Code Navigator is designed for **relationship navigation**:

| Feature | grep/ripgrep | ast-grep | Code Navigator |
|---------|-------------|----------|----------------|
| Find text | ✅ | ✅ | ❌ |
| Parse AST | ❌ | ✅ | ✅ |
| Find callers | ❌ | ❌ | ✅ |
| Trace dependencies | ❌ | ❌ | ✅ |
| Pre-computed graph | ❌ | ❌ | ✅ |
| Token efficient | ❌ | ❌ | ✅ |

**Use grep/ripgrep for**: Finding where text appears in code
**Use ast-grep for**: Structural code search and refactoring
**Use Code Navigator for**: Understanding code relationships and architecture

</details>

<details>
<summary><b>Can I use this with my existing tools?</b></summary>

Yes! Code Navigator complements existing tools:
- **Export to GraphML/DOT** for visualization in Gephi or Graphviz
- **Export to CSV** for spreadsheet analysis
- **Query from scripts** using the JSON output format
- **Integrate with CI/CD** for automated complexity checks

Example integration:
```bash
# Index codebase in CI
codenav index . --language typescript

# Check for complexity violations
codenav analyze hotspots --threshold 15 || exit 1
```

</details>

<details>
<summary><b>Does it work with monorepos?</b></summary>

Yes! Code Navigator handles monorepos efficiently:
- Index each project/module separately
- Use `--exclude` to skip irrelevant directories
- Use `--incremental` for fast updates when only a few files change

Example for a monorepo:
```bash
# Index each package separately
codenav index ./packages/frontend -l typescript -o frontend.bin
codenav index ./packages/backend -l typescript -o backend.bin
codenav index ./packages/shared -l typescript -o shared.bin
```

</details>

<details>
<summary><b>How do I add support for a new language?</b></summary>

Code Navigator uses [tree-sitter](https://tree-sitter.github.io/) for parsing. To add a language:

1. Add the tree-sitter grammar to `Cargo.toml`
2. Create a parser in `src/parser/`
3. Implement the `LanguageParser` trait
4. Add tests with sample code

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed instructions. Contributions welcome!

</details>

<details>
<summary><b>How is this different from using a language server (LSP)?</b></summary>

- **Pre-computed vs On-demand**: Index once, query instantly — no server running per request
- **AI-optimized**: Minimal token output for relationships, not IDE features like completions/hover
- **Portable**: Single `.bin` file — no server connection or session state needed

</details>

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Code standards
- How to add language support
- Pull request process

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Built for AI agents to navigate code at the speed of thought.** ⚡
