# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**beetle** is a blazing-fast source code repository indexing and search tool built with Rust. It uses Tantivy for high-performance full-text search and provides CLI, VS Code extension, and web interfaces for code search across large codebases.

## Development Process

We follow the document first development process, where the design and architecture are documented before implementation. This ensures clarity and alignment on project goals.
We put PRDs (Product Requirements Documents) in the `docs/prd` directory, separate different sections of PRDs into their own files.

## Architecture

- **Core Engine** (`crates/engine/`): Tantivy-based indexing and search functionality
- **CLI Application** (`apps/cli/`): Command-line interface with HTTP API server
- **VS Code Extension** (`editors/vscode/`): TypeScript extension with webviews
- **Web UI** (`apps/webui/`): React-based web interface (Vite + TypeScript)

## Development Commands

### Rust Commands
```bash
# Build entire workspace
cargo build --release

# Build specific packages
cargo build --package beetle --release  # CLI
cargo build --package engine --release  # Core engine

# Run tests
cargo test                              # All tests
cargo test --package engine            # Engine tests only
cargo test --package beetle            # CLI tests only

# Install CLI tool
cargo install --path apps/cli

# Run CLI from source
cargo run --package beetle -- search --index <NAME> --query <QUERY>
```

### Web UI Commands
```bash
cd apps/webui
bun run dev      # Development server on port 3000
bun run build    # Build for production
bun run test     # Run tests
```

### VS Code Extension Commands
```bash
cd editors/vscode
pnpm install     # Install dependencies
pnpm run compile # Build extension
pnpm run test    # Run extension tests
```

## Key Directories

- `~/.beetle/` - Default storage location for indexes
- `apps/cli/tests/fixtures/` - Test data and sample repositories
- `docs/design.md` - Complete technical documentation

## Common Workflows

### Creating and Using Indexes
```bash
# Create index for a project
beetle new --index myproject --path /path/to/project

# Search for code patterns
beetle search --index myproject --query "fn parse"
beetle search --index myproject --query "TODO FIXME" --format json

# Update index with changes
beetle update --index myproject

# List all indexes
beetle list

# Start HTTP API server
beetle serve --port 3000
```

### Development Setup
1. Install Rust toolchain via mise.toml (rust, node, pnpm, bun)
2. Install CLI: `cargo install --path apps/cli`
3. Install VS Code extension dependencies: `cd editors/vscode && pnpm install`
4. Start web UI: `cd apps/webui && npm run dev`

## Technology Stack

- **Core**: Rust 2021, Tantivy search engine, Tokio async runtime
- **CLI**: bpaf argument parser, Axum HTTP server
- **VS Code**: TypeScript, Vue.js webviews, VS Code Extension API
- **Web UI**: React 19, TypeScript, Vite, Tailwind CSS
- **Build**: Cargo workspace with optimized release profiles
