# Code Index Schema Improvement Plan

## Executive Summary

This document outlines a phased approach to enhance the current code index schema. The plan prioritizes features that deliver maximum value early, while laying the groundwork for more advanced code-aware search capabilities. It is designed to be backward compatible, ensuring a smooth transition for existing indexes.

## Current Schema Analysis

### Existing Fields
- `path`: File path (STRING | STORED)
- `content`: File content (ngram3 tokenized, STORED)
- `extension`: File extension (STRING | STORED)
- `last_modified`: Last modification time (u64, FAST | STORED)

### Identified Gaps
1.  **Missing essential metadata** for advanced filtering (e.g., file size, language).
2.  **Inefficient change detection**, relying solely on `last_modified`.
3.  **Limited language support**, based only on file extensions.
4.  **No code-aware search capabilities** (e.g., searching for functions, classes, or TODOs).
5.  **Suboptimal search performance** for common use cases like case-insensitive or fuzzy search.

## Multi-Phase Improvement Plan

### Phase 1: Foundational Metadata (Week 1)
**Priority**: High | **Risk**: Low | **Effort**: 2-3 days

This phase focuses on adding essential metadata to enable more powerful filtering and efficient incremental indexing.

```rust
// Phase 1: New fields
pub content_hash: Field,       // IR-1: SHA-256 hash for reliable change detection
pub language: Field,           // IR-2: Detected programming language
pub file_size: Field,          // SR-2: File size in bytes for filtering
pub line_count: Field,         // SR-2: Line count for filtering
pub file_name: Field,          // SR-2: Filename for targeted searches
pub directory: Field,          // SR-2: Directory path for scoping searches
```

**Benefits**:
-   Reliable and fast change detection using content hashing.
-   Filtering by language, file size, and line count.
-   Searches scoped to specific directories or filenames.

### Phase 2: Enhanced Content and Search (Week 2)
**Priority**: High | **Risk**: Low | **Effort**: 3-4 days

This phase enriches the index with fields derived from file content, improving search relevance and performance.

```rust
// Phase 2: New fields
pub content_lower: Field,      // Case-insensitive search optimization
pub trigram_content: Field,    // Fuzzy and substring search support
pub has_todo: Field,           // Boolean flag for TODO/FIXME comments
pub is_binary: Field,          // Identify binary files
```

**Benefits**:
-   Faster case-insensitive searches.
-   Fuzzy search for handling typos.
-   Quickly find files with TODOs or FIXMEs.
-   Exclude binary files from noisy search results.

## Implementation Details

### Field Configuration

#### `content_hash` & `file_name`
Use a `raw` tokenizer to store the values as-is for exact matching.
```rust
let raw_text_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("raw")
            .set_index_option(IndexRecordOption::Basic)
    )
    .set_stored();
let content_hash = schema_builder.add_text_field("content_hash", raw_text_options.clone());
let file_name = schema_builder.add_text_field("file_name", raw_text_options);
```

#### `file_size` & `line_count`
`u64` fields with `FAST` enabled for efficient range queries.
```rust
// FAST for range queries
let file_size = schema_builder.add_u64_field("file_size", FAST | STORED);
let line_count = schema_builder.add_u64_field("line_count", FAST | STORED);
```

#### `trigram_content`
Use `NgramTokenizer` to create trigrams for fuzzy search.
```rust
let trigram_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("ngram3") // Assuming "ngram3" is a registered NgramTokenizer
            .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    );
let trigram_content = schema_builder.add_text_field("trigram_content", trigram_options);
```

### Language Detection
For the `language` field, we should use a robust library like `golismero-rust-analyzer` or a similar crate that can identify languages based on content, not just file extensions.

### Backward Compatibility Strategy

1.  **Schema Versioning**: Introduce a `schema_version` field in the index's metadata.
2.  **Graceful Degradation**: Existing indexes (version 1) will continue to function. New features will be disabled for these indexes.
3.  **On-the-fly Migration**: When a document is re-indexed (due to a change), populate the new fields. This allows for a gradual, automatic upgrade.
4.  **Explicit Migration**: Provide a `beetle migrate-index` command to allow users to explicitly upgrade an entire index to the new schema version.

### Migration Path

#### For New Indexes
-   New indexes will be created with `schema_version = 2`.
-   All new fields will be populated during the initial indexing.

#### For Existing Indexes
-   Remain at `schema_version = 1`.
-   The `migrate-index` command will re-index all documents, populate the new fields, and update the schema version to 2.

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Breaking existing APIs | High | Low | Maintain a backward compatibility layer in the query engine. |
| Index corruption during migration | High | Very Low | Use transactional commits and write the new index to a temporary location before replacing the old one. |
| Performance degradation | Medium | Low | Benchmark indexing and query performance before and after changes. |
| Increased storage overhead | Low | Medium | The new fields are relatively small compared to the main `content` field. This is an acceptable trade-off for the new features. |

## Success Metrics

### Phase 1 Metrics
-   [ ] `content_hash` based change detection correctly identifies modified files and reduces re-indexing by >90% for unchanged files.
-   [ ] Language detection is >95% accurate on a sample set of diverse repositories.
-   [ ] Queries filtering by `file_size`, `line_count`, and `language` return accurate results.

### Phase 2 Metrics
-   [ ] Case-insensitive searches are at least 2x faster when using the `content_lower` field.
-   [ ] `has:todo` query returns all files containing "TODO" or "FIXME".
-   [ ] Fuzzy search provides relevant results for queries with one or two typos.

## Timeline

| Phase | Duration | Completion Criteria |
|-------|----------|---------------------|
| Phase 1 | 2-3 days | All foundational metadata fields implemented & tested |
| Phase 2 | 3-4 days | Enhanced content/search fields implemented & tested |
| **Total** | **~1 week** | **Phases 1 & 2 complete and deployed** |

## Next Steps
1. **Week 1**: Implement Phase 1  
2. **Week 2**: Implement Phase 2  
3. **Documentation**: Update docs to reflect new schema & search capabilities  
4. **Release**: Ship new version, monitor performance & feedback  

## Appendix: Complete Enhanced Schema (Phases 1 & 2)

```rust
pub struct CodeIndexSchema {
    // Existing fields
    pub schema: Schema,
    pub path: Field,
    pub content: Field,
    pub extension: Field,
    pub last_modified: Field,

    // Phase 1: Foundational Metadata
    pub content_hash: Field,
    pub language: Field,
    pub file_size: Field,
    pub line_count: Field,
    pub file_name: Field,
    pub directory: Field,

    // Phase 2: Enhanced Content and Search
    pub content_lower: Field,
    pub trigram_content: Field,
    pub has_todo: Field,
    pub is_binary: Field,
}
```
| Phase 4 | Future | To be planned after the initial release. |

## Next Steps

1.  **Week 1**: Implement Phase 1.
2.  **Week 2**: Implement Phase 2.
3.  **Documentation**: Update all relevant documentation to reflect the new schema and search capabilities.
4.  **Release**: Release the new version and monitor for performance and feedback.
5.  **Plan Phase 4**: Begin detailed planning for the Code Intelligence phase.

## Appendix: Complete Enhanced Schema (Phases 1 & 2)

```rust
pub struct CodeIndexSchema {
    // Existing fields
    pub schema: Schema,
    pub path: Field,
    pub content: Field,
    pub extension: Field,
    pub last_modified: Field,
    
    // Phase 1: Foundational Metadata
    pub content_hash: Field,
    pub language: Field,
    pub file_size: Field,
    pub line_count: Field,
    pub file_name: Field,
    pub directory: Field,
    
    // Phase 2: Enhanced Content and Search
    pub content_lower: Field,
    pub trigram_content: Field,
    pub has_todo: Field,
    pub is_binary: Field,
}
```
