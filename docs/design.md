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
- **🔄 Incremental Updates**: Efficiently handle file changes with smart incremental indexing (planned)
- **🗑️ Delete Index**: Remove existing indexes when no longer needed
- **♻️ Reindex**: Update an existing index to reflect the latest code changes (planned)

### 2. Search Capabilities
- **🔍 Full-Text Search**: Query indexed repositories using natural language search terms
- **⚡ Fast Retrieval**: Leverage optimized Tantivy indexing for sub-second search results
- **📄 Code Snippets**: Extract and display relevant code snippets with context
- **📊 Multiple Output Formats**: Support for both human-readable text and machine-readable JSON output
- **🎯 Relevance Scoring**: Advanced relevance scoring to surface the most relevant results first

### 3. Developer Integration
- **💻 CLI Interface**: Comprehensive command-line tool for all indexing and search operations
- **🔌 VS Code Extension**: Rich VS Code integration with search panels, index management, and result navigation
- **🔧 Configurable**: Customizable storage locations and indexing options

## Command Reference

beetle provides a comprehensive command-line interface with the following commands:

| Command | Description | Status |
|---------|-------------|---------|
| `new` | Create a new index for a specified directory | ✅ Implemented |
| `search` | Search within an existing index | ✅ Implemented |
| `list` | Display all available indexes | ✅ Implemented |
| `delete` | Remove an index from the system | ✅ Implemented |
| `update` | Update an existing index with incremental or full reindex | 🚧 Planned |

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
beetle delete --index old-project

# Planned: Incremental update
beetle update --index my-project --incremental

# Planned: Full reindex
beetle update --index my-project --reindex
```

## Development Workflow

### Typical User Journey

1. **Initial Setup**: User installs beetle CLI and/or VS Code extension
2. **Index Creation**: Create an index for their active project or multiple projects
3. **Daily Usage**: Use search functionality to find code patterns, functions, TODOs, etc.
4. **Index Maintenance**: Periodically update indexes as codebases evolve (planned)
5. **Project Management**: Create/delete indexes as projects are added/removed

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

- **Core Language**: Rust 2021 Edition
- **Search Engine**: [Tantivy](https://github.com/quickwit-oss/tantivy) - Rust full-text search library
- **CLI Framework**: [bpaf](https://github.com/pacak/bpaf) - Declarative command-line parser
- **VS Code Extension**: TypeScript with VS Code Extension API
- **Serialization**: serde for JSON handling
- **Error Handling**: anyhow for comprehensive error context

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
│   └── cli/                    # Main CLI application
│       ├── src/
│       │   ├── main.rs        # Entry point
│       │   ├── command/       # Command definitions
│       │   └── ...
│       └── Cargo.toml
├── crates/
│   └── beetle_engine/         # Core search engine
│       ├── src/
│       │   ├── lib.rs         # Public API
│       │   ├── index_manager.rs
│       │   ├── search.rs
│       │   └── ...
│       └── Cargo.toml
├── editors/
│   └── vscode/               # VS Code extension
│       ├── src/
│       │   └── extension.ts  # Extension entry point
│       └── package.json
└── docs/
    └── design.md            # This document
```

## Testing Strategy

### Test Categories

**Unit Tests**: Core functionality in `beetle_engine`
- Index creation and management
- Search result ranking and formatting
- Document processing and schema validation

**Integration Tests**: CLI command functionality
- End-to-end command execution
- Cross-platform path handling
- Output format validation

**End-to-End Tests**: Real-world scenarios
- Complete user workflows
- Performance benchmarks
- Edge case handling

**VS Code Extension Tests**: Extension-specific functionality
- Command registration and execution
- UI component behavior
- CLI integration

### Test Data

Test fixtures include sample repositories with:
- Multiple programming languages
- Various file sizes and structures
- Unicode content and special characters
- Common code patterns and edge cases

## Future Roadmap

### Near-term Enhancements (Next 3-6 months)

- **✅ Core MVP**: Basic indexing and search (COMPLETED)
- **🚧 Update Commands**: Incremental and full reindex functionality
- **🚧 Enhanced VS Code Integration**: Better result navigation and previews
- **📋 Configuration System**: User-configurable indexing options
- **📋 Ignore Patterns**: Support for `.gitignore` and custom ignore files

### Medium-term Goals (6-12 months)

- **🔄 Smart Updates**: Automatic file system watching and incremental updates
- **🎯 Advanced Query Syntax**: Support for field-specific queries and filters
- **📊 Search Analytics**: Query history and usage statistics
- **🔌 Editor Integrations**: Support for IntelliJ IDEA, Neovim, Emacs
- **🌐 Web Interface**: Browser-based search interface for teams

This design document serves as both a technical specification and a development guide, ensuring that beetle remains focused on its core mission of providing fast, reliable code search while maintaining a clear path for future enhancements.

# Todo List

## VSCode Extension

- [ ] columns are resizable
- [ ] can run query from multiple indexes
- [ ] search results and the corresponding query are saved in the history and can be reused

- [ ] Highlight matching terms in search results