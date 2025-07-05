use crate::schema::CodeIndexSchema;
use tantivy::schema::Value;
use tantivy::snippet::SnippetGenerator;

use tantivy::{Index, TantivyDocument};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SearchResultItem {
    pub path: String,
    pub snippet: String,
    pub extension: String,
    pub score: f32,
}

impl SearchResultItem {}

pub struct IndexSearcher {
    index: Index,
    reader: tantivy::IndexReader,
}

impl IndexSearcher {
    pub fn new(index: Index) -> Result<Self, String> {
        let reader = index
            .reader()
            .map_err(|e| format!("Failed to create index reader for index: {}", e))?;

        Ok(IndexSearcher { index, reader })
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResultItem>, String> {
        let code_index_schema = CodeIndexSchema::new();

        let query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                code_index_schema.path,
                code_index_schema.content,
                code_index_schema.extension,
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
            SnippetGenerator::create(&searcher, &parsed_query, code_index_schema.content).unwrap();

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher
                .doc::<TantivyDocument>(doc_address)
                .map_err(|e| format!("Failed to retrieve document: {}", e))?;

            let path = doc
                .get_first(code_index_schema.path)
                .unwrap()
                .as_str()
                .unwrap();
            let snippet = snippet_generator.snippet_from_doc(&doc);
            let extension = doc
                .get_first(code_index_schema.extension)
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
