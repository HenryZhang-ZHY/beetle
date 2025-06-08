mod document;
mod index_manager;
mod query;
mod utils;

pub use document::Document;
pub use index_manager::{IndexManager, IndexMetadata};
pub use query::{SearchOptions, SearchResult};

use anyhow::Result;
use std::path::PathBuf;

/// Create a new search index from a repository
pub fn create_index(
    index_name: &str,
    repo_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<String> {
    let manager = IndexManager::new(output_path.clone());
    manager.create_index(index_name, repo_path)
}

/// Search an existing index
pub fn search_index(index_name: &str, query_str: &str) -> Result<String> {
    let manager = IndexManager::default();
    let options = SearchOptions::default();
    manager.search(index_name, query_str, options)
}

/// List all available indexes
pub fn list_indexes() -> Result<String> {
    let manager = IndexManager::default();
    manager.list_indexes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(utils::format_size(512), "512 B");
        assert_eq!(utils::format_size(1024), "1.0 KB");
        assert_eq!(utils::format_size(1536), "1.5 KB");
        assert_eq!(utils::format_size(1048576), "1.0 MB");
        assert_eq!(utils::format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_extract_snippet() {
        let text = "This is a long piece of text that contains the word function somewhere in the middle and we want to extract a snippet around it.";
        let query = "function";
        let snippet = utils::extract_snippet(text, query, 100);

        assert!(
            snippet.contains("function"),
            "Snippet should contain the query word"
        );
        assert!(snippet.len() <= 110, "Snippet should be reasonably sized");
    }
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

    /// Search an in-memory index for testing
    pub fn search_memory_index(index: &Index, query_str: &str) -> Result<String> {
        let searcher = query::create_searcher(index)?;
        let options = query::SearchOptions::default();
        let results = query::search(index, &searcher, query_str, options)?;

        if results.is_empty() {
            return Ok(format!("No results found for query: '{}'", query_str));
        }

        let mut output = format!(
            "Found {} results for query '{}':\n\n",
            results.len(),
            query_str
        );

        for result in results {
            output.push_str(&format!(
                "📄 {} (score: {:.2})\n   Path: {}\n\n",
                result.title, result.score, result.path
            ));
        }

        Ok(output)
    }

    #[test]
    fn test_in_memory_index_creation() {
        let index = create_memory_index();
        assert!(index.is_ok(), "Should be able to create in-memory index");
    }

    #[test]
    fn test_in_memory_index_with_documents() {
        let test_files = vec![
            (
                "main.rs",
                "src/main.rs",
                "fn main() { println!(\"Hello, world!\"); }",
            ),
            (
                "lib.rs",
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a + b }",
            ),
            (
                "test.rs",
                "tests/test.rs",
                "use mylib::add; #[test] fn test_add() { assert_eq!(add(2, 3), 5); }",
            ),
        ];

        let index = create_memory_index_with_documents(test_files);
        assert!(
            index.is_ok(),
            "Should be able to create index with documents"
        );

        if let Ok(index) = index {
            let search_result = search_memory_index(&index, "main");
            assert!(search_result.is_ok(), "Should be able to search index");

            if let Ok(result) = search_result {
                assert!(
                    result.contains("main"),
                    "Search result should contain 'main'"
                );
            }
        }
    }
}
