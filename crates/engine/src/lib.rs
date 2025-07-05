mod document;
mod index_catalog;
mod index_manager;
mod index_storage;
mod index_writter;
mod schema;
mod search;

pub use document::Document;
pub use index_manager::{IndexInfo, IndexManager, IndexMetadata, IndexingOptions, IndexingStats};
pub use search::SearchResult;

use anyhow::Result;
use std::path::PathBuf;

/// Create a new search index from a repository
pub fn new_index(
    index_name: &str,
    path_to_be_indexed: &PathBuf,
    index_path: &PathBuf,
    options: IndexingOptions,
) -> Result<IndexingStats> {
    let manager = IndexManager::new(index_path.clone());
    let mut stats = manager.new_index(index_name, path_to_be_indexed, Some(options))?;

    // Update stats with the actual paths used
    stats.index_name = index_name.to_string();
    stats.index_path = index_path.join(index_name);
    stats.repo_path = path_to_be_indexed.clone();

    Ok(stats)
}

// Add test_utils module at the end of lib.rs
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use anyhow::Context;
    use tantivy::directory::RamDirectory;
    use tantivy::schema::{Schema, STORED, TEXT};
    use tantivy::{doc, Index, IndexWriter};

    /// Create an in-memory index for testing purposes
    pub fn create_memory_index() -> Result<Index> {
        let mut schema_builder = Schema::builder();
        let _title = schema_builder.add_text_field("title", TEXT | STORED);
        let _body = schema_builder.add_text_field("body", TEXT | STORED);
        let _path = schema_builder.add_text_field("path", STORED);
        let schema = schema_builder.build();

        let directory = RamDirectory::create();
        Index::create(directory, schema, Default::default())
            .with_context(|| "Failed to create in-memory tantivy index")
    }

    /// Create an in-memory index with sample documents for testing
    pub fn create_memory_index_with_documents(files: Vec<(&str, &str, &str)>) -> Result<Index> {
        let index = create_memory_index()?;
        let schema = index.schema();
        let title = schema.get_field("title").unwrap();
        let body = schema.get_field("body").unwrap();
        let path_field = schema.get_field("path").unwrap();

        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        for (file_title, file_path, content) in files {
            index_writer.add_document(doc!(
                title => file_title,
                body => content,
                path_field => file_path,
            ))?;
        }

        index_writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_in_memory_index_creation() {
        let index = create_memory_index();
        assert!(index.is_ok(), "Should be able to create in-memory index");
    }
}
