# beetle - Source Code Repository Indexing Tool

## Overview

**beetle** is a blazing-fast source code repository indexing and search tool designed to help developers quickly find code snippets, functions, and patterns across large codebases. Built with Rust for maximum performance and reliability, beetle combines the power of full-text search with intelligent code-aware indexing.

## Key Benefits

- **⚡ Lightning Fast Search**: Powered by [Tantivy](https://github.com/quickwit-oss/tantivy), a high-performance full-text search engine library
- **🧠 Intelligent Code Navigation**: Quickly locate functions, classes, variables, and patterns across large codebases
- **⏱️ Time Efficiency**: Drastically reduce time spent searching for code references and implementations
- **📈 Scalability**: Handle repositories of any size with optimized indexing and efficient memory usage
- **🔧 Developer-Friendly**: Intuitive CLI and VS Code extension for seamless integration into developer workflows
- **🚀 Cross-Platform**: Works seamlessly on Windows, macOS, and Linux

## Core Features

### 1. Index Management
- **🆕 Create Index**: Generate a searchable index for any specified directory with intelligent file type detection
- **📋 List Indexes**: Display all available indexes with their metadata and statistics
- **🔄 Incremental Updates**: Efficiently handle file changes with smart incremental indexing
- **🗑️ Delete Index**: Remove existing indexes when no longer needed
- **♻️ Reindex**: Update an existing index to reflect the latest code changes
- **🌐 HTTP API**: RESTful API server for remote access and integration

### 2. Search Capabilities
- **🔍 Full-Text Search**: Query indexed repositories using natural language search terms
- **⚡ Fast Retrieval**: Leverage optimized Tantivy indexing for sub-second search results
- **📄 Code Snippets**: Extract and display relevant code snippets with context
- **📊 Multiple Output Formats**: Support for both human-readable text and machine-readable JSON output
- **🎯 Relevance Scoring**: Advanced relevance scoring to surface the most relevant results first

### 3. Developer Integration
- **💻 CLI Interface**: Comprehensive command-line tool for all indexing and search operations
- **🔌 VS Code Extension**: Rich VS Code integration with search panels, index management, and result navigation
- **🌐 Web Interface**: Vue.js-based web UI for browser-based search and exploration
- **🔧 Configurable**: Customizable storage locations and indexing options

## Command Reference

beetle provides a comprehensive command-line interface with the following commands:

| Command | Description | Status |
|---------|-------------|---------|
| `new` | Create a new index for a specified directory | ✅ Implemented |
| `search` | Search within an existing index | ✅ Implemented |
| `list` | Display all available indexes | ✅ Implemented |
| `remove` | Remove an index from the system | ✅ Implemented |
| `update` | Update an existing index with incremental or full reindex | ✅ Implemented |
| `serve` | Start HTTP API server for remote access | ✅ Implemented |

### Command Usage Examples

```bash
# Create an index for a repository
beetle new --index my-project --path /path/to/repo

# Search for functions containing "parse"
beetle search --index my-project --query "fn parse"

# Search with JSON output for tooling integration
beetle search --index my-project --query "Result Err" --format json

# List all available indexes
beetle list

# Delete an index when no longer needed
beetle remove --index old-project

# Incremental update (only new/changed files)
beetle update --index my-project

# Full reindex (rebuild entire index)
beetle update --index my-project --reindex

# Start HTTP API server
beetle serve --port 3000
```

## Development Workflow

### Typical User Journey

1. **Initial Setup**: User installs beetle CLI and/or VS Code extension
2. **Index Creation**: Create an index for their active project or multiple projects
3. **Daily Usage**: Use search functionality to find code patterns, functions, TODOs, etc.
4. **Index Maintenance**: Periodically update indexes as codebases evolve (planned)
5. **Project Management**: Create/remove indexes as projects are added/removed

### Search Use Cases

**Function Discovery**:
```bash
beetle search --index myproject --query "fn parse_json"
beetle search --index myproject --query "function parseJSON"
```

**Error Handling Patterns**:
```bash
beetle search --index myproject --query "Result Err unwrap"
beetle search --index myproject --query "try catch exception"
```

**TODO Management**:
```bash
beetle search --index myproject --query "TODO FIXME HACK"
```

**API Usage**:
```bash
beetle search --index myproject --query "http client request"
```

## Implementation Details

### Technology Stack

**Core Engine**:
- **Language**: Rust 2021 Edition
- **Search Engine**: [Tantivy](https://github.com/quickwit-oss/tantivy) - Rust full-text search library
- **File Discovery**: [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) crate for `.gitignore` support
- **Serialization**: serde for JSON handling
- **Error Handling**: anyhow for comprehensive error context

**CLI Application**:
- **CLI Framework**: [bpaf](https://github.com/pacak/bpaf) - Declarative command-line parser
- **HTTP Server**: [axum](https://github.com/tokio-rs/axum) - Modern web framework (planned)
- **Async Runtime**: [tokio](https://tokio.rs/) for async operations

**VS Code Extension**:
- **Language**: TypeScript with VS Code Extension API
- **Build System**: esbuild for fast compilation
- **UI Framework**: Vue.js for webview components

**Web UI**:
- **Frontend**: Vue.js 3 with TypeScript
- **Build Tool**: Vite for fast development and building
- **Testing**: Vitest and Playwright for comprehensive testing
- **Styling**: Modern CSS with component-based architecture

### Key Design Decisions

**Tantivy Choice**: Selected over Elasticsearch/Solr for:
- Zero external dependencies
- Rust-native integration
- Excellent performance for code search use cases
- Manageable memory footprint

**CLI-First Architecture**: 
- VS Code extension as thin wrapper ensures consistency
- Enables easy integration with other editors/tools
- Supports automation and scripting scenarios

**File-Based Storage**:
- No database server required
- Easy backup and migration
- Portable across systems

### Project Structure

```
beetle/
├── apps/
│   ├── cli/                    # Main CLI application
│   │   ├── src/
│   │   │   ├── main.rs        # Entry point
│   │   │   ├── lib.rs         # Library interface
│   │   │   ├── command.rs     # Command definitions
│   │   │   ├── runner.rs      # Runner trait
│   │   │   ├── result.rs      # Result types
│   │   │   ├── server.rs      # HTTP server (planned)
│   │   │   └── command/       # Individual command implementations
│   │   │       ├── new.rs     # Index creation
│   │   │       ├── search.rs  # Search functionality
│   │   │       ├── list.rs    # List indexes
│   │   │       ├── remove.rs  # Remove indexes
│   │   │       ├── update.rs  # Update indexes
│   │   │       ├── serve.rs   # HTTP server
│   │   │       └── formatter/ # Output formatters
│   │   └── Cargo.toml
│   └── webui/                  # Vue.js web interface
│       ├── src/
│       ├── components.json
│       ├── package.json
│       └── vite.config.ts
├── crates/
│   └── engine/                 # Core search engine
│       ├── src/
│       │   ├── lib.rs         # Public API
│       │   ├── catalog.rs     # Index catalog management
│       │   ├── search.rs      # Search implementation
│       │   ├── storage.rs     # Storage abstraction
│       │   ├── schema.rs      # Tantivy schema definition
│       │   ├── writter.rs     # Index writing
│       │   └── file_status_index.rs # File change tracking
│       └── Cargo.toml
├── editors/
│   └── vscode/                # VS Code extension
│       ├── src/
│       │   ├── extension.ts   # Extension entry point
│       │   ├── commands.ts    # Command implementations
│       │   ├── searchEditor.ts # Search editor webview
│       │   └── treeProviders.ts # Tree view providers
│       └── package.json
└── docs/
    └── design.md              # This document
```

## Testing Strategy

### Test Categories

**Unit Tests**: Core functionality in `engine` crate
- Index creation and management
- Search result ranking and formatting
- Document processing and schema validation
- File change tracking and incremental updates

**Integration Tests**: CLI command functionality in `apps/cli/tests/`
- End-to-end command execution
- Cross-platform path handling
- Output format validation (text and JSON)
- Index lifecycle management

**E2E Tests**: Real-world scenarios using `assert_cmd`
- Complete user workflows
- Search result validation
- Error handling and edge cases
- Performance benchmarks

**VS Code Extension Tests**: Extension-specific functionality
- Command registration and execution
- Webview component behavior
- CLI integration and communication
- Tree view providers

**Web UI Tests**: Frontend application testing
- Component unit tests with Vitest
- End-to-end testing with Playwright
- Cross-browser compatibility
- User interaction workflows

### Test Data and Fixtures

Test fixtures are located in `apps/cli/tests/fixtures/` and include:
- Sample repositories with multiple programming languages
- Various file sizes and structures
- Unicode content and special characters
- Common code patterns and edge cases
- Binary files and unsupported formats

## Future Roadmap

### Near-term Enhancements (Next 3-6 months)

- **✅ Core MVP**: Basic indexing and search (COMPLETED)
- **✅ Update Commands**: Incremental and full reindex functionality (COMPLETED)
- **✅ HTTP API Server**: RESTful API for remote access (COMPLETED)
- **🚧 Enhanced VS Code Integration**: Better result navigation and previews
- **🚧 Web UI Polish**: Enhanced Vue.js interface with advanced features
- **📋 Configuration System**: User-configurable indexing options
- **📋 Ignore Patterns**: Support for `.gitignore` and custom ignore files

### Medium-term Goals (6-12 months)

- **🔄 Smart Updates**: Automatic file system watching and incremental updates
- **🎯 Advanced Query Syntax**: Support for field-specific queries and filters
- **📊 Search Analytics**: Query history and usage statistics
- **🔌 Editor Integrations**: Support for IntelliJ IDEA, Neovim, Emacs
- **🌐 Enhanced Web Interface**: Advanced search features and collaboration tools

This design document serves as both a technical specification and a development guide, ensuring that beetle remains focused on its core mission of providing fast, reliable code search while maintaining a clear path for future enhancements.

# Todo List

## CLI Application

- [x] Core indexing and search functionality
- [x] Index management (create, list, remove)
- [x] Update command with incremental and full reindex options
- [x] JSON and plain text output formats
- [x] HTTP API server command
- [ ] Configuration file support
- [ ] Advanced query syntax
- [ ] Progress indicators for long operations

## VSCode Extension

- [x] Basic extension structure and commands
- [x] Index management integration
- [x] Search editor webview
- [x] Tree view for indexes
- [ ] Select the first index by default when opening the search panel
- [ ] Jump to the correct position when opening a file from search results
- [x] Use Vue for the UI
- [ ] Columns are resizable
- [ ] Can run query from multiple indexes
- [ ] Search results and the corresponding query are saved in the history and can be reused
- [ ] Highlight matching terms in search results

## Web UI

- [x] Vue.js-based web interface
- [x] Basic search functionality
- [ ] Advanced search features
- [ ] Result visualization
- [ ] Index management interface
- [ ] Real-time search suggestions

## Engine

- [x] Tantivy integration
- [x] File type detection and indexing
- [x] Search result ranking
- [x] Incremental indexing
- [x] Index catalog management
- [ ] File watching for automatic updates
- [ ] Custom field queries
- [ ] Search result caching

## Workspace Structure and Build System

beetle uses a Cargo workspace to manage multiple related packages:

### Workspace Configuration
- **Root**: Cargo workspace with optimized release profiles
- **Members**: `apps/cli` and `crates/engine`
- **Build Optimization**: LTO, single codegen unit, and size optimization for releases

### Package Dependencies
- **CLI depends on Engine**: The CLI application uses the engine crate as a local dependency
- **Shared Dependencies**: Common dependencies like `anyhow`, `serde`, and `serde_json`
- **Engine-specific**: Tantivy and ignore crate for core search functionality
- **CLI-specific**: bpaf for argument parsing, axum/tokio for HTTP server

### Build Commands
```bash
# Build entire workspace
cargo build --release

# Build specific package
cargo build --package beetle --release
cargo build --package engine --release

# Run tests
cargo test                    # All tests
cargo test --package engine  # Engine tests only
cargo test --package beetle  # CLI tests only

# Install CLI tool
cargo install --path apps/cli
```

### Development Workflow
1. **Engine Development**: Core search functionality in `crates/engine`
2. **CLI Development**: Command-line interface in `apps/cli`
3. **VS Code Extension**: TypeScript development in `editors/vscode`
4. **Web UI**: Vue.js development in `apps/webui`
