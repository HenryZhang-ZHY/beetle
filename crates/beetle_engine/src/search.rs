use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, Searcher};

use crate::document::Document;
use crate::schema::IndexSchema;

use crate::index_manager::IndexManager;

pub struct SearchResult {
    pub path: String,
    pub snippet: String,
    pub score: f32,
}

impl IndexManager {
    fn create_searcher(index: &Index) -> Result<Searcher> {
        let reader: IndexReader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .with_context(|| "Failed to create index reader")?;

        Ok(reader.searcher())
    }

    pub fn search(&self, query_str: &str) -> Result<Vec<SearchResult>> {
        let index = Index::open_in_dir(&self.index_path)
            .with_context(|| format!("Failed to open index at: {}", self.index_path.display()))?;

        let searcher = IndexManager::create_searcher(&index)?;
        let schema = index.schema();
        let path_field = schema.get_field(IndexSchema::PATH_FIELD)?;
        let content_field = schema.get_field(IndexSchema::CONTENT_FIELD)?;

        let query_parser = QueryParser::for_index(&index, vec![path_field, content_field]);
        let query = query_parser
            .parse_query(query_str)
            .with_context(|| format!("Failed to parse query: '{}'", query_str))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(100000))
            .with_context(|| "Failed to execute search")?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher
                .doc(doc_address)
                .with_context(|| "Failed to retrieve document")?;

            let document = Document::from_tantivy_doc(&retrieved_doc, &schema)?;
            let snippet = extract_snippet(&document.content, query_str, 100);

            results.push(SearchResult {
                path: document.path,
                snippet,
                score,
            });
        }

        Ok(results)
    }
}

pub fn extract_snippet(text: &str, query: &str, max_length: usize) -> String {
    let query_words: Vec<&str> = query
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();

    if query_words.is_empty() || text.is_empty() {
        return truncate_text(text, max_length);
    }

    if let Some((pos, word_len)) = find_best_match_position(text, &query_words) {
        extract_snippet_around_position(text, pos, word_len, max_length)
    } else {
        truncate_text(text, max_length)
    }
}

/// Find the best position to extract a snippet from
fn find_best_match_position(text: &str, query_words: &[&str]) -> Option<(usize, usize)> {
    let text_lower = text.to_lowercase();
    let mut best_pos = None;
    let mut best_word_len = 0;

    for word in query_words {
        let word_lower = word.to_lowercase();
        if let Some(pos) = text_lower.find(&word_lower) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
                best_word_len = word.len();
            }
        }
    }

    best_pos.map(|pos| (pos, best_word_len))
}

fn extract_snippet_around_position(
    text: &str,
    pos: usize,
    word_len: usize,
    max_length: usize,
) -> String {
    let context_size = (max_length - word_len) / 2;
    let start = pos.saturating_sub(context_size);
    let end = (pos + word_len + context_size).min(text.len());

    let mut snippet = text[start..end].to_string();
    snippet = clean_snippet(&snippet);

    let prefix = if start > 0 { "..." } else { "" };
    let suffix = if end < text.len() { "..." } else { "" };

    format!("{}{}{}", prefix, snippet.trim(), suffix)
}

fn clean_snippet(snippet: &str) -> String {
    snippet
        .replace('\n', " ")
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        format!("{}...", &text[..max_length].trim())
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_snippet_1() {
        let text = "This is a long piece of text that contains the word function somewhere in the middle and we want to extract a snippet around it.";
        let query = "function";
        let snippet = extract_snippet(text, query, 100);

        assert!(
            snippet.contains("function"),
            "Snippet should contain the query word"
        );
        assert!(snippet.len() <= 110, "Snippet should be reasonably sized");
    }

    #[test]
    fn test_extract_snippet_2() {
        let text = "This is a long piece of text that contains the word function somewhere in the middle and we want to extract a snippet around it.";
        let query = "function";
        let snippet = extract_snippet(text, query, 50);

        assert!(snippet.contains("function"));
        assert!(snippet.starts_with("..."));
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn test_clean_snippet() {
        let input = "This  has\t\ttabs\nand\n\nnewlines    and     spaces";
        let expected = "This has tabs and newlines and spaces";
        assert_eq!(clean_snippet(input), expected);
    }
}
