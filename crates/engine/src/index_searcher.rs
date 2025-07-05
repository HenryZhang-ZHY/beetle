use crate::index_storage::{IndexStorage, IndexStorageMetadata};
use crate::schema::CodeIndexSchema;
use tantivy::schema::Value;
use tantivy::snippet::SnippetGenerator;

use tantivy::{Index, TantivyDocument};

pub struct SearchResultItem {
    pub path: String,
    pub snippet: String,
    pub extension: String,
    pub score: f32,
}

impl SearchResultItem {}

pub struct IndexSearcher<'a> {
    storage: &'a dyn IndexStorage,
    index_metadata: IndexStorageMetadata,
    index: Index,
    reader: tantivy::IndexReader,
}

impl<'a> IndexSearcher<'a> {
    pub fn new(
        storage: &'a dyn IndexStorage,
        index_metadata: IndexStorageMetadata,
        index: Index,
    ) -> Result<Self, String> {
        let reader = index.reader().map_err(|e| {
            format!(
                "Failed to create index reader for index {}: {}",
                index_metadata.index_name, e
            )
        })?;

        Ok(IndexSearcher {
            storage,
            index_metadata,
            index: index,
            reader,
        })
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResultItem>, String> {
        let schema = CodeIndexSchema::create();
        let content_field = schema
            .get_field(CodeIndexSchema::CONTENT_FIELD)
            .map_err(|e| format!("Failed to get content field: {}", e))?;

        let query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                schema.get_field(CodeIndexSchema::PATH_FIELD).unwrap(),
                content_field,
                schema.get_field(CodeIndexSchema::EXTENSION_FIELD).unwrap(),
            ],
        );
        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| format!("Failed to parse query '{}': {}", query, e))?;

        let searcher = self.reader.searcher();
        let top_docs = searcher
            .search(&parsed_query, &tantivy::collector::TopDocs::with_limit(10))
            .map_err(|e| format!("Search failed: {}", e))?;

        let snippet_generator =
            SnippetGenerator::create(&searcher, &*&parsed_query, content_field).unwrap();

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher
                .doc::<TantivyDocument>(doc_address)
                .map_err(|e| format!("Failed to retrieve document: {}", e))?;

            let path = doc
                .get_first(schema.get_field(CodeIndexSchema::PATH_FIELD).unwrap())
                .unwrap()
                .as_str()
                .unwrap();
            let snippet = snippet_generator.snippet_from_doc(&doc);
            let extension = doc
                .get_first(schema.get_field(CodeIndexSchema::EXTENSION_FIELD).unwrap())
                .unwrap()
                .as_str()
                .unwrap();
            let score = _score;

            results.push(SearchResultItem {
                path: path.to_string(),
                snippet: snippet.to_html().to_string(),
                extension: extension.to_string(),
                score,
            });
        }

        Ok(results)
    }
}
