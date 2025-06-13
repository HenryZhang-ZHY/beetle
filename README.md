# beetle 🪲

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)

**beetle** is a blazing-fast source code repository indexing tool that helps developers quickly find code snippets, functions, and patterns across large codebases. Built with Rust for maximum performance and reliability.

## ✨ Features

- **Lightning Fast Search** - Powered by [Tantivy](https://github.com/quickwit-oss/tantivy), a full-text search engine library inspired by Apache Lucene
- **Code-Aware Indexing** - Intelligently indexes source code files while respecting common ignore patterns
- **Cross-Platform** - Works seamlessly on Windows, macOS, and Linux
- **Simple CLI** - Intuitive command-line interface for seamless integration into developer workflows
- **Snippet Extraction** - Shows relevant code snippets with search term highlighting
- **Incremental Updates** - Efficiently handle file changes with incremental indexing
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

1. **Create an index** for your repository:
```bash
beetle new -i myindex -p /path/to/your/repo
```

2. **Search** your indexed code:
```bash
beetle search -i myindex -q "function_name"
```

3. **List** all available indexes:
```bash
beetle list
```

## 📖 Documentation

### Commands

#### `new` - Create a new index for a specified folder
```bash
beetle new --index <INDEX_NAME> --path <PATH>
```
- `INDEX_NAME`: A unique name for your index
- `--path`: Path to the repository/codebase to index

#### `list` - Display all available indexes
```bash
beetle list
```
Shows all indexes with their metadata stored in `~/.beetle`

#### `search` - Search within an existing index
```bash
beetle search --index <INDEX_NAME> --query <SEARCH_QUERY>
```
- `--index`: Name of the index to search
- `--search`: Your search query (supports multiple terms)

#### `update` - Update an existing index
```bash
# Incremental update (only changed files)
beetle update --index <INDEX_NAME> --incremental

# Full reindex
beetle update --index <INDEX_NAME> --reindex
```
- `--incremental`: Update only changed files since last index
- `--reindex`: Rebuild the entire index from scratch

#### `delete` - Remove an index from the system
```bash
beetle delete --index <INDEX_NAME>
```

### Examples

```bash
# Create an index for a Rust project
beetle new --index rust-std --path /path/to/rust/project

# Search for specific functions
beetle search --index rust-std --query "fn parse"

# Search for error handling patterns
beetle search --index rust-std --query "Result Err unwrap"

# Find TODO comments
beetle search --index rust-std --query "TODO FIXME"

# Update index with recent changes
beetle update --index rust-std --incremental

# Remove an old index
beetle delete --index old-project
```

## 🏗️ Architecture

beetle consists of two main components:

- **`beetle_engine`** - Core indexing and search functionality
  - Document parsing and indexing
  - Query processing and result ranking
  - Index management
  - Incremental update handling
  
- **`beetle` CLI** - Command-line interface
  - User-friendly commands
  - Output formatting
  - Cross-platform path handling

## 📁 Storage and Configuration

By default, beetle stores indexes and metadata in the `~/.beetle` directory:

```
~/.beetle/
├── indexes/
│   ├── my-index-01/
│   └── rust-std/
├── metadata/
│   ├── my-index-01.json
│   └── rust-std.json
└── config.json
```

You can customize this location using the `BEETLE_HOME` environment variable:
```bash
export BEETLE_HOME=/custom/path/to/beetle
```

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
RUST_LOG=debug cargo run -- query --index myindex --search "test"

# Build release version
cargo build --release
```

## 🧪 Testing

beetle includes comprehensive test coverage:

- Unit tests for core functionality
- Integration tests for CLI commands
- Edge case testing for special characters and Unicode
- Performance benchmarks

Run tests with:
```bash
cargo test
```

## 📊 Performance

beetle is designed for speed and efficiency:

- Indexes millions of lines of code in seconds
- Search results returned in milliseconds
- Memory-efficient streaming for large files
- Parallel processing for multi-core systems
- Incremental updates minimize re-indexing time

## 🔧 Configuration

beetle works out of the box with sensible defaults. The following file types are automatically indexed:

- Programming languages: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`
- Web: `.html`, `.css`, `.jsx`, `.tsx`, `.vue`
- Configuration: `.json`, `.yaml`, `.toml`, `.xml`
- Documentation: `.md`, `.txt`, `.rst`
- Scripts: `.sh`, `.bash`, `.ps1`, `.bat`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Tantivy](https://github.com/quickwit-oss/tantivy) - A full-text search engine library
- Thanks to all contributors who help make beetle better!

## 🐛 Bug Reports

Found a bug? Please open an issue with:
- Your operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce the bug
- Expected vs actual behavior

## 🚦 Roadmap

- [ ] IDE extensions (VS Code, IntelliJ)
- [ ] Advanced query syntax and filters
- [ ] Web UI for search visualization
- [ ] Support for more file types and languages
- [ ] Search history and saved queries

---

**beetle** - Making code search fast and delightful 🪲✨
