# Beetle 🪲

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)

**Beetle** is a blazing-fast source code search tool that helps developers quickly find code snippets, functions, and patterns across large codebases. Built with Rust for maximum performance and reliability.

## ✨ Features

- **Lightning Fast Search** - Powered by [Tantivy](https://github.com/quickwit-oss/tantivy), a full-text search engine library inspired by Apache Lucene
- **Code-Aware Indexing** - Intelligently indexes source code files while respecting common ignore patterns
- **Cross-Platform** - Works seamlessly on Windows, macOS, and Linux
- **Simple CLI** - Intuitive command-line interface for creating indexes and searching
- **Snippet Extraction** - Shows relevant code snippets with search term highlighting
- **Multiple Index Support** - Manage multiple indexes for different projects

## 🚀 Quick Start

### Installation

```bash
# Install from source (requires Rust toolchain)
git clone https://github.com/yourusername/beetle.git
cd beetle
cargo install --path apps/cli
```

### Basic Usage

1. **Create an index** for your codebase:
```bash
beetle create myproject -p /path/to/your/project -o /path/to/store/index
```

2. **Search** your indexed code:
```bash
beetle search myproject -q "function parseJSON"
```

3. **List** all available indexes:
```bash
beetle list
```

## 📖 Documentation

### Commands

#### `create` - Create a new search index
```bash
beetle create <INDEX_NAME> -p <REPO_PATH> -o <OUTPUT_PATH>
```
- `INDEX_NAME`: A unique name for your index
- `-p, --path`: Path to the repository/codebase to index
- `-o, --output`: Directory where index files will be stored

#### `search` - Search an existing index
```bash
beetle search <INDEX_NAME> -q <QUERY>
```
- `INDEX_NAME`: Name of the index to search
- `-q, --query`: Your search query (supports multiple terms)

#### `list` - Show all available indexes
```bash
beetle list
```

### Examples

```bash
# Index a Rust project
beetle create rust-std -p ~/rust/library -o ~/.beetle/indexes

# Search for specific functions
beetle search rust-std -q "fn parse"

# Search for error handling patterns
beetle search myproject -q "Result Err unwrap"

# Find TODO comments
beetle search myproject -q "TODO FIXME"
```

## 🏗️ Architecture

Beetle consists of two main components:

- **`beetle_engine`** - Core indexing and search functionality
  - Document parsing and indexing
  - Query processing and result ranking
  - Index management
  
- **`beetle` CLI** - Command-line interface
  - User-friendly commands
  - Output formatting
  - Cross-platform path handling

## 🤝 Contributing

We welcome contributions! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/beetle.git
cd beetle

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- search myindex -q "test"

# Build release version
cargo build --release
```

## 🧪 Testing

Beetle includes comprehensive test coverage:

- Unit tests for core functionality
- Integration tests for CLI commands
- Edge case testing for special characters and Unicode
- Performance benchmarks

Run tests with:
```bash
cargo test
```

## 📊 Performance

Beetle is designed for speed and efficiency:

- Indexes millions of lines of code in seconds
- Search results returned in milliseconds
- Memory-efficient streaming for large files
- Parallel processing for multi-core systems

## 🔧 Configuration

Beetle works out of the box with sensible defaults. The following file types are automatically indexed:

- Programming languages: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`
- Web: `.html`, `.css`, `.jsx`, `.tsx`, `.vue`
- Configuration: `.json`, `.yaml`, `.toml`, `.xml`
- Documentation: `.md`, `.txt`, `.rst`
- Scripts: `.sh`, `.bash`, `.ps1`, `.bat`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Tantivy](https://github.com/quickwit-oss/tantivy) - A full-text search engine library
- Thanks to all contributors who help make Beetle better!

## 🐛 Bug Reports

Found a bug? Please open an issue with:
- Your operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce the bug
- Expected vs actual behavior

## 🚦 Roadmap

- [ ] IDE extensions (VS Code, IntelliJ)
- [ ] Incremental index updates
- [ ] Search history and saved queries

---

**Beetle** - Making code search fast and delightful 🪲✨
