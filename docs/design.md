# beetle - Source Code Repository Indexing Tool

## Overview

beetle is a command-line interface (CLI) tool designed to efficiently index and query source code repositories. It provides developers with powerful search capabilities across their codebase, enabling faster code discovery and analysis.

## Benefits

- **Improved Code Navigation**: Quickly locate functions, classes, and patterns across large codebases
- **Time Efficiency**: Reduce time spent searching for code references
- **Scalability**: Handle repositories of various sizes with optimized indexing
- **Simplicity**: Intuitive command-line interface for seamless integration into developer workflows

## Core Features

### 1. Index Management
- **Create Index**: Generate a searchable index for any specified directory
- **List Indexes**: Display all available indexes with their metadata
- **Incremental Updates**: Support for incremental indexing to handle file changes efficiently
- **Delete Index**: Remove existing indexes when no longer needed
- **Reindex**: Update an existing index to reflect the latest code changes

### 2. Query Capabilities
- **Search Functionality**: Query indexed repositories using various search parameters
- **Fast Retrieval**: Leverage indexed data for rapid search results
- **Flexible Queries**: Support for different query types and filters

## Command Structure

The following commands are available:

| Command | Description |
|---------|-------------|
| `create` | Create a new index for a specified folder |
| `list` | Display all available indexes |
| `query` | Search within an existing index |
| `delete` | Remove an index from the system |
| `update` | Update an existing index with new changes or reindex |

## Usage Examples

```bash
# Create an index for a repository
beetle create my-index-01 --path /path/to/repository

# List all indexes
beetle list

# Query an existing index
beetle query --index my-index-01 --search "function_name"

# Incremental update an index
beetle update --index my-index-01 --incremental

# Delete an index
beetle delete --index my-index-01

# Reindex to update with latest changes
beetle update --index my-index-01 --reindex
```

## Storage and Metadata

By default, beetle stores indexes and metadata in the `~/.beetle` directory. This location can be customized using the `BEETLE_HOME` environment variable.

## Structure of beetle folder

The `~/.beetle` directory contains the following structure:

```~/.beetle/
├── indexes/
│   ├── my-index-01/
├── metadata/
│   ├── my-index-01.json
└── config.json
```
