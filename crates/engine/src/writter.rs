use crate::change::{diff_file_index_metadata, scan};
use crate::schema::{CodeIndexDocument, CodeIndexSchema};
use crate::storage::{IndexStorage, IndexStorageMetadata};
use rayon::prelude::*;
use std::time::Instant;
use tantivy::{Index, TantivyDocument};
use tracing::{info, span, Level};

pub struct IndexWriter<'a> {
    storage: &'a dyn IndexStorage,
    index_metadata: IndexStorageMetadata,
    writer: tantivy::IndexWriter,
}

impl<'a> IndexWriter<'a> {
    pub fn new(
        storage: &'a dyn IndexStorage,
        index_metadata: IndexStorageMetadata,
        index: Index,
    ) -> Result<Self, String> {
        let writer = index.writer(4 * 1024 * 1024 * 1024).map_err(|e| {
            format!(
                "Failed to create index writer for index {}: {}",
                index_metadata.index_name, e
            )
        })?;

        Ok(IndexWriter {
            storage,
            index_metadata,
            writer,
        })
    }

    pub fn index(&mut self) -> Result<(), String> {
        let _span = span!(Level::INFO, "index_writer_index",
            index_name = %self.index_metadata.index_name,
            target_path = %self.index_metadata.target_path
        )
        .entered();

        let start_time = Instant::now();

        let file_index_snapshot = self
            .storage
            .read_file_index_metadata(&self.index_metadata.index_name)?;
        info!(
            "loaded file index snapshot with {} files",
            file_index_snapshot.len()
        );

        let manifest = scan(&self.index_metadata.target_path);
        info!("scanned current file index with {} files", manifest.len());

        let delta = diff_file_index_metadata(&file_index_snapshot, &manifest);
        info!(
            files_added = delta.added.len(),
            files_modified = delta.modified.len(),
            files_removed = delta.removed.len(),
            "calculated file delta"
        );

        let code_index_schema = CodeIndexSchema::new();
        let removed = delta.removed;
        let removal_start = Instant::now();
        for file in removed {
            let file_path = file.path.clone();
            self.writer.delete_term(tantivy::Term::from_field_text(
                code_index_schema.path,
                &file_path,
            ));
        }
        let removal_duration = removal_start.elapsed();
        info!(
            duration_ms = removal_duration.as_millis(),
            "completed file removals"
        );

        let files_to_update: Vec<_> = delta.added.into_iter().chain(delta.modified).collect();
        let total_files = files_to_update.len();

        const BATCH_SIZE: usize = 100;
        let batch_count = total_files.div_ceil(BATCH_SIZE);
        let processing_start = Instant::now();

        for (batch_idx, batch) in files_to_update.chunks(BATCH_SIZE).enumerate() {
            let batch_span = span!(
                Level::INFO,
                "process_batch",
                batch_index = batch_idx,
                batch_size = batch.len(),
                total_batches = batch_count
            );
            let _batch_guard = batch_span.enter();

            let batch_start = Instant::now();

            let documents: Result<Vec<_>, _> = batch
                .par_iter()
                .map(|file| -> Result<TantivyDocument, String> {
                    let document = CodeIndexDocument::from_path(&file.path);
                    Ok(document.to_tantivy_document(&code_index_schema.schema))
                })
                .collect();

            let doc_creation_duration = batch_start.elapsed();

            let add_start = Instant::now();
            for doc in documents? {
                self.writer.add_document(doc).map_err(|e| {
                    format!(
                        "Failed to add document to index {}: {}",
                        self.index_metadata.index_name, e
                    )
                })?;
            }
            let add_duration = add_start.elapsed();
            let total_batch_duration = batch_start.elapsed();

            info!(
                batch_size = batch.len(),
                doc_creation_ms = doc_creation_duration.as_millis(),
                doc_add_ms = add_duration.as_millis(),
                total_batch_ms = total_batch_duration.as_millis(),
                files_per_sec = (batch.len() as f64 / total_batch_duration.as_secs_f64()) as u64,
                "completed batch processing"
            );
        }

        let processing_duration = processing_start.elapsed();

        let commit_start = Instant::now();
        self.writer.commit().map_err(|e| {
            format!(
                "Failed to commit index writer for index {}: {}",
                self.index_metadata.index_name, e
            )
        })?;
        let commit_duration = commit_start.elapsed();

        self.storage
            .save_file_index_metadata(&self.index_metadata.index_name, manifest)?;

        let total_duration = start_time.elapsed();

        info!(
            total_files = total_files,
            total_duration_ms = total_duration.as_millis(),
            processing_duration_ms = processing_duration.as_millis(),
            commit_duration_ms = commit_duration.as_millis(),
            files_per_sec = if total_duration.as_secs_f64() > 0.0 {
                (total_files as f64 / total_duration.as_secs_f64()) as u64
            } else {
                0
            },
            docs_per_sec = if processing_duration.as_secs_f64() > 0.0 {
                (total_files as f64 / processing_duration.as_secs_f64()) as u64
            } else {
                0
            },
            "indexing completed"
        );

        Ok(())
    }
}
