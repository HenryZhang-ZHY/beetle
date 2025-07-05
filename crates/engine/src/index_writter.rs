use crate::file_scanner::{self, FileScanner};
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
        let document = CodeIndexDocument::new(
            "example/path/to/file.rs".to_string(),
            "fn main() { println!(\"Hello, world!\"); }".to_string(),
            "rs".to_string(),
            std::time::SystemTime::now(),
        );

        let tantivy_doc = document.to_tantivy_document(&CodeIndexSchema::create());

        self.writer.add_document(tantivy_doc).map_err(|e| {
            format!(
                "Failed to add document to index {}: {}",
                self.index_metadata.index_name, e
            )
        })?;

        self.writer.commit().map_err(|e| {
            format!(
                "Failed to commit index writer for index {}: {}",
                self.index_metadata.index_name, e
            )
        })?;

        let file_scanner = FileScanner {};
        let files = file_scanner.scan(&self.index_metadata.target_path);

        Ok(())
    }
}
