mod document;
mod formatter;
mod index_manager;
mod query;
mod schema;
mod utils;

pub use document::Document;
pub use formatter::{JsonFormatter, PlainTextFormatter, ResultFormatter};
pub use index_manager::{IndexInfo, IndexManager, IndexMetadata, IndexingOptions, IndexingStats};
pub use query::{QueryOptions, QueryResult};

use anyhow::Result;
use std::path::PathBuf;

/// Create a new search index from a repository
pub fn new_index<F: ResultFormatter>(
    index_name: &str,
    path_to_be_indexed: &PathBuf,
    index_path: &PathBuf,
    options: IndexingOptions,
    formatter: &F,
) -> Result<String> {
    let manager = IndexManager::new(index_path.clone());
    let mut stats = manager.new_index(index_name, path_to_be_indexed, Some(options))?;

    // Update stats with the actual paths used
    stats.index_name = index_name.to_string();
    stats.index_path = index_path.join(index_name);
    stats.repo_path = path_to_be_indexed.clone();

    formatter.format_indexing_stats(&stats)
}

/// Search an existing index
pub fn search_index<F: ResultFormatter>(
    index_name: &str,
    query_str: &str,
    options: QueryOptions,
    formatter: &F,
) -> Result<String> {
    let manager = IndexManager::default();
    let results = manager.search(index_name, query_str, options)?;
    formatter.format_search_results(query_str, &results)
}

/// List all available indexes
pub fn list_indexes<F: ResultFormatter>(formatter: &F) -> Result<String> {
    let manager = IndexManager::default();
    let indexes = manager.list_indexes()?;
    formatter.format_index_list(&indexes)
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
    pub fn search_memory_index(index: &Index, query_str: &str) -> Result<Vec<QueryResult>> {
        let searcher = query::create_searcher(index)?;
        let options = query::QueryOptions::default();
        query::search(index, &searcher, query_str, options)
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

        let index = create_memory_index_with_documents(test_files).unwrap();
        let results = search_memory_index(&index, "main").unwrap();

        assert!(!results.is_empty(), "Should find results for 'main'");
        assert!(
            results[0].snippet.contains("main"),
            "Snippet should contain 'main'"
        );
    }

    #[test]
    fn test_formatters() {
        let results = vec![QueryResult {
            path: "src/test.rs".to_string(),
            score: 0.95,
            snippet: "fn test() { ... }".to_string(),
        }];

        // Test plain text formatter
        let plain_formatter = PlainTextFormatter;
        let plain_output = plain_formatter
            .format_search_results("test", &results)
            .unwrap();
        assert!(plain_output.contains("test.rs"));
        assert!(plain_output.contains("score: 0.95"));

        // Test JSON formatter
        let json_formatter = JsonFormatter::new(false);
        let json_output = json_formatter
            .format_search_results("test", &results)
            .unwrap();
        assert!(json_output.contains("\"query\":\"test\""));
        assert!(json_output.contains("\"count\":1"));
    }

    #[test]
    fn test_api_with_formatters() {
        // Test list_indexes with different formatters
        let plain_formatter = PlainTextFormatter;
        let json_formatter = JsonFormatter::new(false);

        // These tests would require actual indexes to exist
        // Just testing that the API compiles correctly
        let _ = list_indexes(&plain_formatter);
        let _ = list_indexes(&json_formatter);
    }
}
