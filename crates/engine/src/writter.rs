use crate::file_status_index::{diff_file_index_metadata, FileScanner};
use crate::schema::{CodeIndexDocument, CodeIndexSchema};
use crate::storage::{IndexStorage, IndexStorageMetadata};
use rayon::prelude::*;
use tantivy::{Index, TantivyDocument};
use tracing::trace;


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
        let file_index_snapshot = self
            .storage
            .read_file_index_metadata(&self.index_metadata.index_name)?;
        trace!(
            "loaded file index snapshot with {} files",
            file_index_snapshot.len()
        );

        let file_scanner = FileScanner {};
        let manifest = file_scanner.scan(&self.index_metadata.target_path);
        trace!("scanned current file index with {} files", manifest.len());

        let delta = diff_file_index_metadata(&file_index_snapshot, &manifest);
        trace!(
            "calculated delta: added {}, modified {}, removed {}",
            delta.added.len(),
            delta.modified.len(),
            delta.removed.len()
        );

        let code_index_schema = CodeIndexSchema::new();
        let removed = delta.removed;
        for file in removed {
            let file_path = file.path.clone();
            self.writer.delete_term(tantivy::Term::from_field_text(
            code_index_schema.path,
            &file_path,
            ));
        }

        let files_to_update: Vec<_> = delta.added.into_iter().chain(delta.modified).collect();

        const BATCH_SIZE: usize = 100;        
        for batch in files_to_update.chunks(BATCH_SIZE) {
            let documents: Result<Vec<_>, _> = batch
                .par_iter()
                .map(|file| -> Result<TantivyDocument, String> {
                    let document = CodeIndexDocument::from_path(&file.path);
                    Ok(document.to_tantivy_document(&code_index_schema.schema))
                })
                .collect();

            for doc in documents? {
                self.writer.add_document(doc).map_err(|e| {
                    format!(
                        "Failed to add document to index {}: {}",
                        self.index_metadata.index_name, e
                    )
                })?;
            }
        }

        self.writer.commit().map_err(|e| {
            format!(
                "Failed to commit index writer for index {}: {}",
                self.index_metadata.index_name, e
            )
        })?;

        self.storage
            .save_file_index_metadata(&self.index_metadata.index_name, manifest)?;

        Ok(())
    }
}
