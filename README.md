# beetle 🪲

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)

**beetle** is a blazing-fast source code repository indexing and search tool that helps developers quickly find code snippets, functions, and patterns across large codebases. Built with Rust for maximum performance and reliability.

> 📖 **For detailed technical documentation, architecture, and design decisions, see [docs/design.md](docs/design.md)**

## ✨ Key Features

- **⚡ Lightning Fast** - Sub-second search results powered by [Tantivy](https://github.com/quickwit-oss/tantivy)
- **🧠 Code-Aware** - Intelligent indexing with support for 50+ file types
- **🚀 Cross-Platform** - Works on Windows, macOS, and Linux
- **💻 Developer-Friendly** - Simple CLI + VS Code extension
- **📊 Flexible Output** - Human-readable text or JSON for tooling integration

## 🚀 Quick Start

### Installation

```bash
# Install from source (requires Rust toolchain)
git clone https://github.com/yourusername/beetle.git
cd beetle
cargo install --path apps/cli
```

### Basic Usage

```bash
# 1. Create an index for your repository
beetle new --index myproject --path /path/to/your/repo

# 2. Search your code
beetle search --index myproject --query "function_name"

# 3. List all indexes
beetle list
```

### Common Search Patterns

```bash
# Find functions
beetle search --index myproject --query "fn parse"

# Find error handling
beetle search --index myproject --query "Result Err unwrap"

# Find TODOs
beetle search --index myproject --query "TODO FIXME"

# Get JSON output for tooling
beetle search --index myproject --query "main" --format json
```

## 📖 Command Reference

| Command | Description | Status |
|---------|-------------|---------|
| `new` | Create a new index | ✅ |
| `search` | Search within an index | ✅ |
| `list` | List all indexes | ✅ |
| `delete` | Delete an index | ✅ |
| `update` | Update an index | 🚧 Planned |

### Command Examples

```bash
# Create index
beetle new --index <NAME> --path <PATH>

# Search (text output)
beetle search --index <NAME> --query <QUERY>

# Search (JSON output)
beetle search --index <NAME> --query <QUERY> --format json

# List indexes
beetle list

# Delete index
beetle delete --index <NAME>
```

> 📖 **For detailed command documentation and usage examples, see [docs/design.md](docs/design.md)**

## 🏗️ Architecture & Storage

**Architecture**: beetle uses a layered design with a core engine (`engine`), CLI tool, and VS Code extension.

**Storage**: Indexes are stored in `~/.beetle/` by default. Customize with the `BEETLE_HOME` environment variable.

```
~/.beetle/
├── indexes/
│   ├── project-1/
│   └── project-2/
└── metadata/
```

**Supported File Types**: 50+ file types including Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, HTML, CSS, JSON, Markdown, and more.

> 📖 **For complete architecture details, see [docs/design.md](docs/design.md)**

## 🛠️ Development

### Setup

```bash
git clone https://github.com/yourusername/beetle.git
cd beetle
cargo test        # Run tests
cargo build       # Build debug
cargo build --release  # Build release
```

### Testing

```bash
cargo test                           # All tests
cargo test --package engine  # Engine tests only
cargo test --package beetle         # CLI tests only
```

> 📖 **For detailed development guidelines, testing strategy, and contribution guide, see [docs/design.md](docs/design.md)**

## 📊 Performance & Configuration

**Performance Highlights**:
- Sub-second search results
- Indexes millions of lines in seconds  
- Memory-efficient streaming
- Parallel processing support

**Auto-detected File Types**: Programming languages (Rust, Python, JS/TS, Go, Java, C/C++), web files, configs, docs, and more.

**Roadmap**: VS Code extension, incremental updates, advanced query syntax, web UI.

> 📖 **For performance benchmarks and detailed roadmap, see [docs/design.md](docs/design.md)**

## 📄 License & Support

**License**: MIT License - see [LICENSE](LICENSE) file

**Acknowledgments**: Built with [Tantivy](https://github.com/quickwit-oss/tantivy) search engine

---

**beetle** - Making code search fast and delightful 🪲✨
