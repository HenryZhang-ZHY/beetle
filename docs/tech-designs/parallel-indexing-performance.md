# Tech Design: Parallel Indexing Performance Improvement

## Problem Statement

The current `IndexWriter::index()` method in `crates/engine/src/writter.rs` processes files sequentially, which fails to utilize available hardware resources effectively. This creates a bottleneck when indexing large codebases with thousands of files.

### Current Performance Issues

1. **Sequential Processing**: Files are processed one-by-one in a single thread (`writter.rs:66-75`)
2. **File I/O Bottleneck**: Reading and processing files sequentially doesn't leverage modern multi-core CPUs
3. **Limited Tantivy Writer Utilization**: The Tantivy writer's multi-threading capabilities are underutilized
4. **No Parallel Document Processing**: Document creation and indexing operations are performed serially

## Solution: Parallel Processing with Rayon

### High-Level Strategy

Implement parallel processing using the `rayon` crate to distribute file processing across available CPU cores while maintaining thread safety with Tantivy's index writer.

### Architecture Changes

#### 1. Parallel Document Processing
```rust
use rayon::prelude::*;

// Replace sequential for-loop with parallel iterator
let files_to_update: Vec<_> = delta.added.into_iter().chain(delta.modified).collect();

// Process files in parallel to create documents
let documents: Result<Vec<_>, _> = files_to_update
    .par_iter()
    .map(|file| {
        let document = CodeIndexDocument::from_path(&file.path);
        Ok(document.to_tantivy_document(&code_index_schema.schema))
    })
    .collect();

// Add documents sequentially (IndexWriter is NOT thread-safe)
let docs = documents?;
for doc in docs {
    self.writer.add_document(doc)?;
}
```

#### 2. Thread-Safe Index Writing Strategy
```rust
// IMPORTANT: Tantivy's IndexWriter is NOT thread-safe for concurrent writes
// We need to either:
// 1. Process documents in parallel, then write sequentially
// 2. Use a mutex to synchronize writes
// 3. Use a channel to send documents to a single writer thread

// Option 1: Parallel processing + sequential writing (recommended)
let documents = process_files_parallel(files)?;
for doc in documents {
    self.writer.add_document(doc)?;
}

// Option 2: Mutex-protected writer (less efficient)
let writer = Arc::new(Mutex::new(&mut self.writer));
files.par_iter().try_for_each(|file| {
    let doc = create_document(file)?;
    writer.lock().unwrap().add_document(doc)?;
    Ok::<_, String>(())
})?;
```

#### 3. Batch Processing Strategy
```rust
// Process files in chunks to balance parallelism with memory usage
const BATCH_SIZE: usize = 100; // Reduced from 1000 to manage memory better

files_to_update
    .chunks(BATCH_SIZE)
    .try_for_each(|batch| -> Result<(), String> {
        // Process batch in parallel to create documents
        let documents: Result<Vec<_>, _> = batch
            .par_iter()
            .map(|file| {
                let document = CodeIndexDocument::from_path(&file.path);
                Ok(document.to_tantivy_document(&code_index_schema.schema))
            })
            .collect();
        
        // Add documents sequentially
        for doc in documents? {
            self.writer.add_document(doc)
                .map_err(|e| format!("Failed to add document: {}", e))?;
        }
        
        Ok(())
    })?;
```

### Implementation Details

#### Phase 1: Basic Parallel Processing
- [x] Add `rayon` dependency to `Cargo.toml`
- [x] Implement parallel document creation using `par_iter()`
- [x] Maintain sequential writing to IndexWriter (not thread-safe)
- [x] Implement proper error handling with context preservation

#### Phase 2: Indexing Performance Observability
- [x] Integrate the `tracing` crate for structured, async-aware instrumentation  
- [x] Add `tracing::span!` and `tracing::info!` calls inside `IndexWriter::index()` to measure:  
  • overall indexing duration  
  • per-batch latency  
  • docs/sec & files/sec throughput  
  • document creation vs addition timing
  • commit duration tracking

#### Phase 3: Advanced Optimization
- [ ] Implement adaptive batch sizing based on available memory
- [ ] Add memory usage monitoring to prevent OOM
- [ ] Consider using channels for producer-consumer pattern
- [ ] Add configuration for parallel vs sequential modes

#### Phase 4: Performance Monitoring
- [ ] Add metrics collection for indexing performance
- [ ] Implement timing measurements for parallel vs sequential processing
- [ ] Add configuration options for tuning parallelism

### Code Structure Changes

#### Modified `IndexWriter::index()` method:
```rust
pub fn index(&mut self) -> Result<(), String> {
    let _span = span!(Level::INFO, "index_writer_index", 
        index_name = %self.index_metadata.index_name,
        target_path = %self.index_metadata.target_path.display()
    ).entered();
    
    let start_time = Instant::now();
    
    // ... existing setup code ...
    
    let files_to_update: Vec<_> = delta.added.into_iter().chain(delta.modified).collect();
    
    // Process files in batches to manage memory usage
    const BATCH_SIZE: usize = 100;
    
    for (batch_idx, batch) in files_to_update.chunks(BATCH_SIZE).enumerate() {
        let batch_span = span!(Level::INFO, "process_batch", 
            batch_index = batch_idx,
            batch_size = batch.len()
        );
        let _batch_guard = batch_span.enter();
        
        let batch_start = Instant::now();
        
        // Process batch in parallel to create documents
        let documents: Result<Vec<_>, _> = batch
            .par_iter()
            .map(|file| -> Result<tantivy::Document, String> {
                let document = CodeIndexDocument::from_path(&file.path);
                Ok(document.to_tantivy_document(&code_index_schema.schema))
            })
            .collect();
        
        // Add documents sequentially (IndexWriter is not thread-safe)
        for doc in documents? {
            self.writer.add_document(doc)
                .map_err(|e| format!("Failed to add document: {}", e))?;
        }
        
        let batch_duration = batch_start.elapsed();
        info!(
            batch_size = batch.len(),
            duration_ms = batch_duration.as_millis(),
            files_per_sec = (batch.len() as f64 / batch_duration.as_secs_f64()) as u64,
            "completed batch processing"
        );
    }
    
    // ... rest of method ...
}
```

### Performance Expectations

#### Expected Improvements:
- **1.5-3x speedup** on multi-core systems (realistic expectation)
- **CPU-bound scaling** limited by document processing, not I/O
- **Memory-conscious processing** to handle large codebases safely
- **Better CPU utilization** during document creation phase

#### Benchmarking Targets:
- Test with 1,000-10,000 files (more realistic range)
- Measure document creation vs indexing time separately
- Compare memory usage patterns
- Test on systems with 4-16 CPU cores

### Risk Mitigation

#### Thread Safety Considerations:
1. **Tantivy Writer**: Confirmed thread-safe for `add_document()` operations
2. **Error Handling**: Ensure parallel error propagation doesn't lose context
3. **Memory Management**: Implement bounds to prevent memory exhaustion
4. **Progress Reporting**: Add atomic counters for progress tracking

#### Configuration Options:
```rust
#[derive(Debug, Clone)]
pub struct IndexWriterConfig {
    pub enable_parallel_processing: bool,
    pub batch_size: usize,
    pub max_memory_mb: Option<usize>,
    pub num_threads: Option<usize>, // Override rayon's default
}

impl Default for IndexWriterConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: true,
            batch_size: 100,
            max_memory_mb: Some(512), // Reasonable default
            num_threads: None, // Use rayon's default
        }
    }
}
```

### Testing Strategy

#### Unit Tests:
- Test parallel processing with small file sets
- Verify error handling in parallel context
- Test thread safety of document processing

#### Integration Tests:
- Compare performance between sequential and parallel modes
- Test with various file sizes and counts
- Validate index consistency between modes

#### Performance Tests:
- Benchmark with realistic codebase sizes
- Measure CPU and memory usage
- Test scaling behavior with different core counts

### Migration Plan
1. **Phase 1**: Add `rayon` dependency and basic parallel processing (behind feature flag) ✅
2. **Phase 2**: Add `tracing`-based observability for indexing performance ✅
3. **Phase 3**: Enable parallel processing by default with fallback to sequential mode  
4. **Phase 4**: Add configuration options and performance tuning  
5. **Phase 5**: Remove sequential mode if parallel proves stable

### Dependencies

#### New Dependencies:
- `rayon = "1.8"` – parallelism ✅
- `tracing = "0.1"` – structured logging and performance instrumentation ✅
- `num_cpus = "1.16"` – optional CPU detection

#### Cargo.toml updates:
```toml
[dependencies]
rayon     = "1.8"
tracing   = "0.1"
num_cpus  = "1.16"   # Optional: for thread count optimization
```

### Success Criteria

- [ ] 1.5-3x performance improvement on multi-core systems (realistic target)
- [ ] Memory usage remains bounded and predictable
- [ ] Thread-safe operation without data races or deadlocks
- [ ] Graceful degradation on single-core systems
- [ ] Maintained error handling with proper context
- [ ] Zero data corruption or index inconsistency
- [ ] Comprehensive test coverage including edge cases
- [x] Comprehensive performance observability with structured logging
