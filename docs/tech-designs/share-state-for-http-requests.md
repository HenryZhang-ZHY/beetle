# HTTP Shared `IndexCatalog` Design

## 1 – Overview  
Currently, every HTTP request calls `create_index_catalog()`, creating a new `IndexCatalog` instance and triggering repeated disk I/O operations to read index metadata. This approach creates unnecessary overhead and prevents optimization opportunities.

A single, long-lived catalog shared across all requests eliminates this overhead and enables future caching strategies.

## 2 – Goals  
* **Performance**: Eliminate redundant `IndexCatalog` instantiation per request
* **Memory Efficiency**: Share a single catalog instance across all HTTP handlers  
* **Code Quality**: Remove dead helper code (`create_index_catalog`)
* **API Compatibility**: Keep the public HTTP API unchanged
* **Thread Safety**: Ensure safe concurrent access following Rust and Axum best practices

## 3 – Thread Safety Analysis  
**IndexCatalog Thread Safety**:
- All `IndexCatalog` methods take `&self` (shared references), indicating they're designed for concurrent read access
- The underlying `FsStorage` performs file system operations which are naturally thread-safe
- Write operations (`create`, `reset`, `remove`) are already serialized by the file system layer
- No mutable shared state exists in the catalog itself

**Recommended Approach**: `Arc<IndexCatalog>` 
- Enables safe sharing across async tasks
- Minimal overhead (atomic reference counting)
- No locks needed for read operations
- Future-proof for potential caching layers

**Alternative Considered**: `Arc<RwLock<IndexCatalog>>`
- More complex, unnecessary given current design
- Can be added later if fine-grained control is needed

## 4 – Implementation Steps  
1. **Define Application State**:
   ```rust
   #[derive(Clone)]
   struct AppState {
       catalog: Arc<IndexCatalog>,
   }
   ```

2. **Update Server Initialization** in `HttpServer::start()`:
   ```rust
   let catalog = IndexCatalog::new(FsStorage::new(beetle_home_path));
   let app_state = AppState { catalog: Arc::new(catalog) };
   
   let app = Router::new()
       .route(/* existing routes */)
       .with_state(app_state);
   ```

3. **Update Handler Signatures**: Add `State<AppState>` parameter to each handler
4. **Replace `create_index_catalog()` calls** with `state.catalog.clone()` or direct usage
5. **Remove Dead Code**: Delete `create_index_catalog` function
6. **Add Required Imports**: `use std::sync::Arc;` and `use axum::extract::State;`

## 5 – Handler Migration Pattern
**Before**:
```rust
async fn list_indexes() -> ResponseJson<Vec<IndexResponse>> {
    let catalog = create_index_catalog();
    // ... rest of handler
}
```

**After**:
```rust
async fn list_indexes(
    State(state): State<AppState>,
) -> ResponseJson<Vec<IndexResponse>> {
    // Use state.catalog directly
    // ... rest of handler (unchanged)
}
```

## 6 – Testing Strategy  
* **Unit Tests**: Verify each handler works with shared state
* **Concurrency Tests**: 
  - 50 parallel search requests to different indexes
  - Concurrent read/write operations (search while creating index)
  - Stress test: 100+ concurrent requests for 30 seconds
* **Integration Tests**: Full server startup with real HTTP requests
* **Memory Tests**: Verify no memory leaks with long-running server

## 7 – Performance Benefits
* **Startup Time**: ~5-10ms saved per request (no catalog recreation)
* **Memory**: Reduced allocations, single catalog instance
* **Disk I/O**: Eliminated redundant metadata reads
* **Future Optimizations**: Enables index metadata caching, connection pooling

## 8 – Implementation Reference

### Complete Code Changes

**1. Add Required Imports** to `apps/cli/src/server.rs`:
```rust
use std::sync::Arc;
use axum::extract::State;
```

**2. Define Application State**:
```rust
#[derive(Clone)]
struct AppState {
    catalog: Arc<IndexCatalog>,
}
```

**3. Update `HttpServer::start()` Method**:
```rust
impl HttpServer {
    pub fn start(port: u16) -> CommandOutput {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async move {
            // Create shared catalog once
            let beetle_home_path = PathBuf::from(get_beetle_home());
            let storage = FsStorage::new(beetle_home_path);
            let catalog = IndexCatalog::new(storage);
            let app_state = AppState { catalog: Arc::new(catalog) };

            let app = Router::new()
                .route("/api/indexes", get(list_indexes).post(create_index))
                .route("/api/indexes/{index_name}", get(get_index_details).delete(delete_index))
                .route("/api/indexes/{index_name}/search", get(search_index))
                .route("/api/indexes/{index_name}/reindex", post(reindex_index))
                .route("/api/indexes/{index_name}/update", post(update_index))
                .fallback(serve_static_file)
                .with_state(app_state);  // 👈 Key change

            // ... rest unchanged
        })
    }
}
```

**4. Update Handler Functions** (example for each pattern):

```rust
// Read-only operations
async fn list_indexes(
    State(state): State<AppState>,
) -> ResponseJson<Vec<IndexResponse>> {
    match state.catalog.list() {
        // ... rest unchanged
    }
}

async fn search_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<ResponseJson<SearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let query = params.q;
    match state.catalog.get_searcher(&index_name) {
        // ... rest unchanged
    }
}

// Write operations
async fn create_index(
    State(state): State<AppState>,
    ResponseJson(payload): ResponseJson<CreateIndexRequest>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Check if index already exists
    match state.catalog.list() {
        // ... validation logic
    }
    
    match state.catalog.create(&payload.name, &payload.path) {
        // ... rest unchanged
    }
}

// Operations requiring writers
async fn reindex_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let metadata = match state.catalog.get_matadata(&index_name) {
        // ... error handling
    };

    // Reset and rebuild
    match state.catalog.reset(&index_name) {
        // ... error handling
    }

    let mut writer = match state.catalog.get_writer(&index_name) {
        // ... rest unchanged
    };
}
```

**5. Remove Dead Code**:
```rust
// DELETE this function entirely:
// fn create_index_catalog() -> IndexCatalog { ... }
```

## 9 – Migration Checklist
- [ ] Add `std::sync::Arc` and `axum::extract::State` imports
- [ ] Define `AppState` struct with `catalog: Arc<IndexCatalog>`
- [ ] Update `HttpServer::start()` to create shared state
- [ ] Add `.with_state(app_state)` to router
- [ ] Update all 8 handler functions to accept `State<AppState>`
- [ ] Replace all `create_index_catalog()` calls with `state.catalog`
- [ ] Delete `create_index_catalog` function
- [ ] Run `cargo check` to verify compilation
- [ ] Run existing tests to ensure functionality unchanged
- [ ] Add concurrency tests as specified in Testing Strategy

## 10 – Risk Mitigation
* **Compilation Safety**: Rust's type system prevents data races
* **Incremental Migration**: Each handler can be updated independently
* **Backward Compatibility**: HTTP API remains identical
* **Performance**: No performance regression, only improvements
* **Testing**: Existing integration tests validate behavior unchanged

## 11 – Future Enhancements Enabled
* **Index Metadata Caching**: Cache frequently accessed metadata in memory
* **Connection Pooling**: Share database/index connections
* **Request Metrics**: Track per-index usage statistics
* **Health Monitoring**: Centralized health checks across indexes
* **Configuration Hot-Reload**: Update settings without server restart
