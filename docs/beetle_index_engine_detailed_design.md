GitHub Copilot: # Beetle Index Engine – Detailed Design

> Status : Draft  
> Author : o3 (2025-06-29)  

---

## 1 . Goals
* Fast, safe, **incremental** indexing for large source-code repositories.  
* Clear separation of concerns – scan, diff, write, search are independent.  
* Swappable storage back-ends (local FS today, S3/SQLite tomorrow).  
* Single writer / shared readers per Tantivy index to respect locking rules.  
* Minimal memory / handle footprint; aggressive reuse of resources.  

---

## 2 . High-Level Architecture

```mermaid
classDiagram
    %% storage
    class IndexStorage {
        +index_dir()
        +create()
        +open()
        +remove()
        +list()
    }
    class FsStorage { +root: PathBuf }
    IndexStorage <|.. FsStorage

    class ManifestStore { +load() +store() }

    %% incremental chain
    class FileScanner { +scan(repo,opts) Vec~FileMeta~ }
    class FileMeta { path String; mtime u64; size u64 }
    class IndexDiffer { +diff(snapshot,manifest) Delta }
    class Delta { added Vec~FileMeta~; modified Vec~FileMeta~; removed Vec~FileMeta~ }

    class IndexUpdater {
        +update(repo,opts) IndexingStats
        -scanner FileScanner
        -differ  IndexDiffer
        -manifest ManifestStore
        -storage IndexStorage
        -writer  IndexWriter
    }

    %% catalog / factory
    class IndexCatalog {
        +list()
        +reader(name)
        +writer(name,mb)
        +remove(name)
        -storage IndexStorage
        -cache {name→Arc<Index>}
    }

    %% search
    class IndexSearcher { +search(q,limit) Vec~SearchHit~ +reload() }
    class SearchHit { path String; score f32 }

    %% misc
    class IndexingOptions
    class IndexingStats
```

---

## 3 . Component Responsibilities

| Layer | Responsibility | Key Public API |
|-------|----------------|----------------|
| **IndexStorage** | Map “index name” ↔ directory and perform raw file I/O. | `create / open / remove / list / index_dir` |
| **ManifestStore** | Persist `manifest.json` that stores `{mtime,size}` for every indexed file. | `load(name) -> HashMap`, `store(name,snapshot)` |
| **FileScanner** | Walk repository using `ignore::WalkBuilder`; returns `Vec<FileMeta>`. | `scan(repo, opts)` |
| **IndexDiffer** | Compare snapshot vs manifest, produce `Delta`. | `diff(snapshot, manifest)` |
| **IndexUpdater** | Apply `Delta` to Tantivy: delete removed docs, (re)index changed docs, commit, update manifest. | `update(repo, opts)` |
| **IndexCatalog** | Factory + cache: exposes *one* `IndexWriter`, a shared `IndexReader`, and list/remove. | `reader / writer / list / remove` |
| **IndexSearcher** | Read-side façade: executes queries, hides Tantivy internals. | `search`, `reload` |
| **IndexingOptions** | Scanning knobs – gitignore, hidden files, etc. | struct fields + `new()` helpers |

---

## 4 . Incremental Update Algorithm

```text
repo ──► FileScanner ──► snapshot
                    manifest.json ─┐
                                   ▼
                            IndexDiffer ──► Delta
                                   │
                                   ▼
                           IndexUpdater
   Delta.removed   ──► delete_term(path)     \
   Delta.(add|mod) ──► add_document(content)  │ Tantivy writer
                           commit()           /
                           ↓                 /
                 ManifestStore.store(snapshot)
```

Complexity: O(#changed files) disk reads, O(#delta) Tantivy writes.

---

## 5 . Concurrency & Resource Management

* **Writer**: Catalog guarantees 1 `IndexWriter` per index (`once_cell + Mutex`).  
* **Readers**: one shared `IndexReader` cached globally; queries spawn cheap `Searcher`s.  
* **Segment cleanup**: few readers → segments reclaimed quickly after commit+reload.  
* **Thread safety**: All public objects are `Send + Sync` via `Arc`/locks; updater itself is exclusive while running.

---

## 6 . Error Handling

* Library layer returns `anyhow::Error`; CLI maps to user-friendly messages and exit codes.  
* Distinct error variants (`IndexNotFound`, `LockBusy`, `CorruptManifest`) can be promoted later.

---

## 7 . Extensibility Points

| Point | How to extend |
|-------|---------------|
| Storage back-end | Implement `IndexStorage` (e.g. `S3Storage`). |
| Manifest format  | Swap/augment `ManifestStore` (sled, sqlite). |
| File filtering   | Add predicate hooks to `FileScanner`. |
| Ranking / query  | Extend `IndexSearcher` with field boosts, facets, highlights. |

---

## 8 . Migration Plan

1. Introduce new modules alongside existing `IndexManager`.  
2. Adapt CLI to call `IndexCatalog + IndexUpdater + IndexSearcher`.  
3. Deprecate and finally remove `IndexManager`.  

---

## 9 . Future Work

* Hash-based change detection to cover timestamp spoofing.  
* Background merge / optimize scheduler.  
* Distributed sharding (multiple storage roots).  

---

### Appendix A – Important Structs

```rust
pub struct FileMeta { path: String, mtime: u64, size: u64 }
pub struct Delta   { added: Vec<FileMeta>, modified: Vec<FileMeta>, removed: Vec<FileMeta> }
pub struct IndexingStats { file_count: u64, total_size: u64, duration_ms: u64 }
```

### Appendix B - 30-Minute Task Checklist

| # | Task (≈ 30 min each) | Deliverable / Test | Status |
|---|----------------------|--------------------|--------|
| 1 | Create empty `engine` crate/modules; run `cargo check`. | Workspace compiles. | DONE |
| 2 | Implement `schema.rs`: `IndexSchema::create`, `CONTENT_FIELD`, `PATH_FIELD`. | Unit test passes. | DONE |
| 3 | Declare `trait IndexStorage` and stub `FsStorage { root }`. | Compiles. | DONE |
| 4 | Implement `FsStorage::index_dir / create / open`. | `storage_create_open` test. | DONE |
| 5 | Implement `FsStorage::remove / list` with tests. | Tests pass. | DONE |
| 6 | Add `indexing_options.rs` and `is_text_file` helper (serde-ready). | `cargo check` ok. | TODO |
| 7 | Implement `file_scanner.rs` → `FileScanner::scan`. | Snapshot test. | TODO |
| 8 | Implement `manifest_store.rs` (`load / store`). | Round-trip test. | TODO |
| 9 | Implement `index_differ.rs` (`Delta` + `diff`). | Diff unit test. | TODO |
|10 | Skeleton `index_updater.rs` (fields, no Tantivy yet). | Compiles. | TODO |
|11 | Integrate Tantivy in updater: open/create index, writer, fields. | Compile link. | TODO |
|12 | Apply `removed` set using `writer.delete_term`. | Deletion test. | TODO |
|13 | Apply `added/modified` docs; read file, `add_document`. | Incremental update test. | TODO |
|14 | Finish commit + manifest save; return `IndexingStats`. | All updater tests green. | TODO |
|15 | Implement `index_catalog.rs` with caches, `list/reader/writer/remove`. | Multi-thread test. | TODO |
|16 | Swap cache to `dashmap` / optimize; micro-benchmark. | No perf regression. | TODO |
|17 | Implement `index_searcher.rs` (`search`, `reload`). | Query test. | TODO |
|18 | Create `beetle_cli` crate; add `clap` skeleton (`new/update/search/list/remove`). | `beetle --help`. | TODO |
|19 | Wire CLI `new` to `IndexUpdater::update` (initial build). | Manual run ok. | TODO |
|20 | Wire CLI `update` to incremental update. | Modify file → result updated. | TODO |
|21 | Wire CLI `search` to `IndexSearcher`. | Returns paths. | TODO |
|22 | Wire CLI `list/remove` to catalog. | Index dirs change. | TODO |
|23 | Deprecate/remove old `IndexManager`; shim to new impl. | Tests pass. | TODO |
|24 | Update docs (`design.md`, README, Mermaid). | Render OK. | TODO |
|25 | Raise test coverage to > 80 %. | `tarpaulin` report. | TODO |
|26 | Cross-platform path tests (Win/Linux, CR-LF). | CI green. | TODO |
|27 | Benchmark full vs incremental (100k files); doc results. | `docs/benchmark.md`. | TODO |
|28 | Expose writer memory via config/CLI; tune compression. | New bench results. | TODO |
|29 | Add `catalog.optimize(name)` segment merge API. | Manual test. | TODO |
|30 | Prepare release: cargo-release dry run, GitHub Actions workflow. | v0.1.0 tag ready. | TODO |
