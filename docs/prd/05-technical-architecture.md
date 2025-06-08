# Technical Architecture

## System Overview

Beetle is built as a modular, high-performance code search system using Rust as the primary language. The architecture follows a layered design pattern with clear separation of concerns between indexing, storage, search, and presentation layers.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interfaces                          │
├─────────────────┬───────────────────┬───────────────────────┤
│   CLI Tool      │   VS Code Ext     │    Web Interface      │
│  (apps/cli)     │ (editors/vscode)  │   (apps/webui)        │
└─────────────────┴───────────────────┴───────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    API Layer                                │
├─────────────────────────────────────────────────────────────┤
│         HTTP API Server (Axum)                              │
│         RESTful Endpoints                                   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                  Core Engine                                │
├─────────────────────────────────────────────────────────────┤
│         Search Service                                      │
│         Index Management                                    │
│         Query Processing                                    │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                            │
├─────────────────────────────────────────────────────────────┤
│         Tantivy Index Engine                                │
│         File System Storage                                 │
│         Metadata Storage                                    │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Engine Layer (`crates/engine/`)

The engine layer is the heart of Beetle, responsible for all indexing and search operations.

### 2. Storage Architecture

#### 2.1 Index Storage
**Technology**: Tantivy search engine library
**Location**: `~/.beetle/indexes/{index_name}/`

**Index Structure**:
```
index/
├── meta.json          # Index metadata
├── schema.json        # Document schema
├── segments/          # Tantivy segments
├── temp/             # Temporary files
└── lock              # Index lock file
```

**Document Schema**:
```rust
pub struct CodeDocument {
    file_path: String,           // Full file path
    file_name: String,           // File name only
    content: String,             // Full file content
    language: String,            // Programming language
    file_size: u64,              // File size in bytes
    modified_time: DateTime,     // Last modification time
    line_count: u32,             // Number of lines
    extension: String,           // File extension
    directory: String,           // Parent directory
    checksum: String,            // Content checksum
}
```

#### 2.2 Metadata Storage
**Technology**: SQLite for metadata, JSON for configuration
**Purpose**: Store index configuration, usage statistics, and user preferences

**Tables**:
- `indexes`: Index metadata and configuration
- `files`: File metadata and checksums
- `searches`: Search history and analytics
- `settings`: User preferences and system settings

### 3. API Architecture

#### 3.1 HTTP API (`apps/cli/src/server/`)
**Framework**: Axum (async Rust web framework)
**Base URL**: `http://localhost:3000/api/v1`

**Endpoint Structure**:
```
GET    /indexes                    # List all indexes
POST   /indexes                    # Create new index
GET    /indexes/:name              # Get index details
POST   /indexes/:name/search       # Search index
PUT    /indexes/:name              # Update index
DELETE /indexes/:name              # Delete index
GET    /health                     # Health check
```

**Request/Response Formats**:
- JSON for all requests/responses
- Consistent error format
- Pagination support for large result sets
- Filtering and sorting capabilities

### 4. CLI Architecture

#### 4.1 Command Structure
**Framework**: `bpaf` for argument parsing
**Style**: Git-style command interface

**Command Hierarchy**:
```
beetle
├── new      # Create new index
├── search   # Search existing indexes
├── list     # List all indexes
├── update   # Update existing index
├── delete   # Delete index
├── serve    # Start HTTP server
├── status   # Show system status
└── config   # Manage configuration
```

#### 4.2 Output Formats
**Supported Formats**:
- Human-readable tables
- JSON for programmatic use
- CSV for spreadsheet import
- Custom formatting via templates

### 5. VS Code Extension Architecture

#### 5.1 Extension Structure
**Technology**: TypeScript with VS Code Extension API
**Architecture**: WebView-based interface for search results

**Components**:
- **Extension Host**: Main extension logic
- **WebView Panel**: Search interface
- **Language Server**: Optional for advanced features
- **Settings**: VS Code configuration integration

#### 5.2 Communication
- **Extension ↔ Core**: Via CLI commands
- **WebView ↔ Extension**: Message passing API
- **Settings Sync**: Automatic VS Code settings integration

### 6. Web Interface Architecture

#### 6.1 Frontend Stack
**Framework**: React 19 with TypeScript
**Build Tool**: Vite
**Styling**: Tailwind CSS
**State Management**: React Query + Zustand

**Component Structure**:
```
src/
├── components/          # Reusable UI components
├── pages/              # Route-based pages
├── hooks/              # Custom React hooks
├── services/           # API client services
├── stores/             # State management
├── utils/              # Utility functions
└── types/              # TypeScript definitions
```

#### 6.2 Search Interface
**Features**:
- Real-time search with debouncing
- Infinite scroll for results
- Syntax highlighting with Prism.js
- Responsive design for mobile/desktop
- Keyboard shortcuts for power users

### 7. Data Flow Architecture

#### 7.1 Indexing Flow
```
Repository → File Discovery → Language Detection → Content Processing → 
Tokenization → Document Creation → Tantivy Indexing → Index Commit
```

#### 7.2 Search Flow
```
User Query → Query Parsing → Tantivy Search → Result Processing → 
Highlighting → Ranking → Response Formatting → User Display
```

### 8. Concurrent Architecture

#### 8.1 Threading Model
**Tokio Runtime**: Async runtime for I/O operations
**Rayon**: Data parallelism for CPU-intensive tasks

**Thread Pools**:
- **Indexing Pool**: For file processing and indexing
- **Search Pool**: For query execution
- **Network Pool**: For HTTP/web operations

#### 8.2 Async Patterns
- **Stream Processing**: For large file processing
- **Channel Communication**: Between components
- **Cancellation Tokens**: For graceful shutdown
- **Rate Limiting**: For API endpoints

### 10. Extension Architecture

#### 10.1 Plugin System (Future)
**Architecture**: WASM-based plugin system
**Capabilities**:
- Custom analyzers
- Language-specific processors
- Custom search operators
- Integration plugins

#### 10.2 Configuration Extension
- **User Scripts**: Custom processing pipelines
- **Custom Languages**: Support for new programming languages
- **Export Formats**: Custom output formats

### 11. Monitoring and Observability

#### 11.1 Metrics Collection
**Technology**: `prometheus` crate for metrics
**Metrics Tracked**:
- Search query latency
- Indexing throughput
- Error rates
- Resource utilization
- User engagement (opt-in)

#### 11.2 Logging Architecture
**Framework**: `tracing` with structured logging
**Log Levels**:
- ERROR: System failures
- WARN: Recoverable issues
- INFO: Important events
- DEBUG: Development information
- TRACE: Detailed flow

#### 11.3 Distributed Tracing
**Technology**: OpenTelemetry integration
**Use Cases**:
- Performance analysis
- Debugging complex issues
- User experience optimization

### 12. Security Architecture

#### 12.1 Threat Model
**Assets**:
- Source code indexes
- User configuration
- API keys and secrets

**Threats**:
- Unauthorized access to indexes
- Code injection via search queries
- Privilege escalation
- Data exfiltration

#### 12.2 Security Controls
- **Input Sanitization**: All user inputs validated
- **Access Control**: File system permissions
- **Encryption**: Optional for sensitive data
- **Audit Logging**: Security-relevant events

### 13. Deployment Architecture

#### 13.1 Distribution Methods
- **Cargo Install**: Primary distribution via crates.io
- **Package Managers**: Homebrew, Chocolatey, apt (future)
- **Docker**: Container images for consistent deployment
- **Static Binaries**: Single binary with no dependencies

#### 13.2 Update Mechanism
- **Automatic Updates**: Opt-in via CLI
- **Version Checking**: Periodic checks for new versions
- **Rollback Support**: Revert to previous version
- **Compatibility Checks**: Ensure index compatibility

### 14. Testing Architecture

#### 14.1 Test Strategy
**Unit Tests**: Core functionality testing
**Integration Tests**: Component interaction testing
**Performance Tests**: Benchmarking and load testing
**End-to-End Tests**: Complete workflow testing

#### 14.2 Test Data
- **Synthetic Repositories**: Generated test data
- **Real Projects**: Popular open-source projects
- **Edge Cases**: Large files, special characters, etc.
- **Performance Benchmarks**: Standardized datasets

### 15. Development Architecture

#### 15.1 Development Environment
**Tooling**:
- Cargo workspace for multi-crate management
- Pre-commit hooks for code quality
- Automated formatting with rustfmt
- Linting with clippy
- Documentation generation with rustdoc

#### 15.2 Build Pipeline
**Stages**:
1. Code quality checks
2. Unit tests
3. Integration tests
4. Performance benchmarks
5. Cross-platform builds
6. Security scanning
7. Release packaging
