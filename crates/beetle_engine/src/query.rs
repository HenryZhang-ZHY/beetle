use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, Searcher};

use crate::document::Document;
use crate::utils::extract_snippet;

/// Options for search queries
#[derive(Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub snippet_length: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            snippet_length: 100,
        }
    }
}

/// Represents a search result
pub struct SearchResult {
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub score: f32,
}

/// Create a searcher from an index
pub fn create_searcher(index: &Index) -> Result<Searcher> {
    let reader: IndexReader = index
        .reader_builder()
        .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .with_context(|| "Failed to create index reader")?;

    Ok(reader.searcher())
}

/// Execute a search query
pub fn search(
    index: &Index,
    searcher: &Searcher,
    query_str: &str,
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    let schema = index.schema();
    let title_field = schema.get_field("title")?;
    let body_field = schema.get_field("body")?;

    let query_parser = QueryParser::for_index(index, vec![title_field, body_field]);
    let query = query_parser
        .parse_query(query_str)
        .with_context(|| format!("Failed to parse query: '{}'", query_str))?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(options.limit))
        .with_context(|| "Failed to execute search")?;

    let mut results = Vec::new();

    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher
            .doc(doc_address)
            .with_context(|| "Failed to retrieve document")?;

        let document = Document::from_tantivy_doc(&retrieved_doc, &schema)?;
        let snippet = extract_snippet(&document.body, query_str, options.snippet_length);

        results.push(SearchResult {
            title: document.title,
            path: document.path,
            snippet,
            score,
        });
    }

    Ok(results)
}
