use crate::file_scanner::{FileScanner, IndexDiffer};
use crate::index_storage::{IndexStorage, IndexStorageMetadata};
use crate::schema::{CodeIndexDocument, CodeIndexSchema};
use tantivy::Index;

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

        let file_scanner = FileScanner {};
        let manifest = file_scanner.scan(&self.index_metadata.target_path);

        let index_differr = IndexDiffer {};
        let delta = index_differr.diff(&file_index_snapshot, &manifest);

        let schema = CodeIndexSchema::create();
        let path_field = schema
            .get_field(CodeIndexSchema::PATH_FIELD)
            .map_err(|e| format!("Failed to get path field: {}", e))?;

        let removed = delta.removed;
        for file in removed {
            let file_path = file.path.clone();
            self.writer
                .delete_term(tantivy::Term::from_field_text(path_field, &file_path));
        }

        let files_to_update = delta.added.into_iter().chain(delta.modified);
        for file in files_to_update {
            let document = CodeIndexDocument::from_path(&file.path);
            let tantivy_doc = document.to_tantivy_document(&schema);
            self.writer.add_document(tantivy_doc).map_err(|e| {
                format!(
                    "Failed to add document to index {}: {}",
                    self.index_metadata.index_name, e
                )
            })?;
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
