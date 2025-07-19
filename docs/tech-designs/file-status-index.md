# File Status Index

Beetle tracks which files have changed so it can update its search index quickly.  
A compact binary snapshot is used instead of JSON to keep disk-space, I/O and CPU costs low.

## Why a Binary Format?

* No field names or whitespace → smaller on disk.
* Direct byte reads/writes → faster than text parsing.
* Single checksum at the end → quick corruption check.

## What Gets Stored?

For every file we remember:

1. Absolute path (UTF-8).
2. File size (`u64`).
3. Modification time (`u64`, seconds since epoch).

A short header (magic + version + entry count) comes first, the records follow, and an 8-byte CRC-64 footer finishes the file. All numbers are big-endian.

## Implementation (crates/engine/src/change.rs)

* `encode`  
  - Pre-allocates a `Vec<u8>` big enough for header + payload + footer.  
  - Streams bytes directly (no `Cursor`).  
  - Updates CRC64 incrementally while writing.  

* `decode`  
  - Verifies CRC before doing any work.  
  - Reads fixed-width fields by slicing the input buffer.  
  - Creates `&str` slices for paths to avoid extra allocations.

The in-memory struct is:

```rust
pub struct FileIndexMetadata {
    pub path: String,
    pub size: u64,
    pub modified_time: u64,
}
```

## Storage Integration (crates/engine/src/storage.rs)

`FsStorage` keeps a `file_index_snapshot.bin` beside every Tantivy index.

* `save_file_index_metadata` → calls `encode` and writes the bytes.  
* `read_file_index_metadata` → reads the file (if any) and `decode`s it.  
* Missing snapshot = empty vector (first-run friendly).

This snapshot lets Beetle compute a fast delta (`diff_file_index_metadata`) between the previous run and the current filesystem scan, so only added/changed/removed files are re-indexed.
