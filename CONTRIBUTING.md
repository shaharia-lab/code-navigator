# Contributing to Code Navigator

We welcome contributions! Here's how to get started.

## Development Setup

1. **Install Rust** (1.70 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/shaharia-lab/code-navigator.git
   cd code-navigator
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run tests**:
   ```bash
   cargo test
   ```

## Code Standards

- **Format code** before committing:
  ```bash
  cargo fmt
  ```

- **Lint code** with Clippy:
  ```bash
  cargo clippy -- -D warnings
  ```

- **Write tests** for new features and bug fixes

- **Keep it simple**: Avoid over-engineering, prefer clarity over cleverness

## Pull Request Process

1. **Fork** the repository
2. **Create a branch** for your feature: `git checkout -b feature/my-feature`
3. **Make your changes** and commit with clear messages
4. **Add tests** for your changes
5. **Ensure all tests pass**: `cargo test`
6. **Run fmt and clippy**: `cargo fmt && cargo clippy`
7. **Push to your fork**: `git push origin feature/my-feature`
8. **Open a Pull Request** with a clear description of the changes

## What to Contribute

### High Priority

- **New language support**: Java, C++, Ruby, Rust, PHP, C#
- **Performance improvements**: Faster parsing, better memory usage
- **Bug fixes**: Check [issues](https://github.com/shaharia-lab/code-navigator/issues)

### Medium Priority

- **Query language**: DSL for complex graph queries
- **Advanced analysis**: Cyclomatic complexity, dead code detection
- **Documentation**: Examples, tutorials, use cases
- **Tests**: Improve coverage, add integration tests

### Nice to Have

- **Web UI**: Interactive graph visualization
- **IDE plugins**: VS Code, IntelliJ, Vim
- **Export formats**: Additional visualization formats

## Adding Language Support

To add a new language parser:

1. Add tree-sitter grammar to `Cargo.toml`:
   ```toml
   tree-sitter-java = "0.23"
   ```

2. Create parser in `src/parser/`:
   ```rust
   // src/parser/java.rs
   pub struct JavaParser {
       parser: Parser,
   }

   impl LanguageParser for JavaParser {
       fn parse_file(&mut self, content: &str, file_path: &Path) -> Result<Vec<Node>> {
           // Implementation
       }
   }
   ```

3. Update `src/parser/mod.rs` to include new parser

4. Add test fixtures in `tests/fixtures/java/`

5. Add tests in `src/lib.rs`

## Reporting Issues

When reporting bugs, include:
- **Code Navigator version**: `code-navigator --version`
- **Operating system**: Linux, macOS, Windows
- **Language and project size**: e.g., "TypeScript, 5,000 files"
- **Steps to reproduce**: Clear steps to trigger the bug
- **Expected vs actual behavior**: What should happen vs what does happen
- **Error messages**: Full error output if available

## Questions?

Open an [issue](https://github.com/shaharia-lab/code-navigator/issues) or start a [discussion](https://github.com/shaharia-lab/code-navigator/discussions).

## Code of Conduct

- **Be respectful**: Treat others with kindness and professionalism
- **Be constructive**: Provide helpful feedback and suggestions
- **Be inclusive**: Welcome contributors of all backgrounds and skill levels
- **Be patient**: Remember that everyone was a beginner once

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
