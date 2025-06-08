use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::index_manager::{IndexInfo, IndexingStats};
use crate::query::SearchResult;
use crate::utils::format_size;

/// Trait for formatting various results
pub trait ResultFormatter {
    /// Format search results
    fn format_search_results(&self, query: &str, results: &[SearchResult]) -> Result<String>;

    /// Format index list
    fn format_index_list(&self, indexes: &[IndexInfo]) -> Result<String>;

    /// Format indexing stats
    fn format_indexing_stats(&self, stats: &IndexingStats) -> Result<String>;
}

/// Plain text formatter (human-readable)
pub struct PlainTextFormatter;

impl ResultFormatter for PlainTextFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResult]) -> Result<String> {
        if results.is_empty() {
            return Ok(format!("No results found for query: '{}'", query));
        }

        let mut output = format!("Found {} results for query '{}':\n\n", results.len(), query);

        for result in results {
            output.push_str(&format!(
                "📄 {} (score: {:.2})\n   Path: {}\n   Preview: {}\n\n",
                result.title, result.score, result.path, result.snippet
            ));
        }

        Ok(output)
    }

    fn format_index_list(&self, indexes: &[IndexInfo]) -> Result<String> {
        if indexes.is_empty() {
            return Ok("No indexes found. Create one with: beetle create <index_name> -p <repo_path> -o <output_path>".to_string());
        }

        let mut result = format!("Found {} index(es):\n\n", indexes.len());

        for index in indexes {
            result.push_str(&format!(
                "📂 {}\n   Path: {}\n   Documents: {}\n   Size: {}\n\n",
                index.name,
                index.path.display(),
                index.metadata.doc_count,
                format_size(index.metadata.size_bytes)
            ));
        }

        Ok(result)
    }

    fn format_indexing_stats(&self, stats: &IndexingStats) -> Result<String> {
        Ok(format!(
            "Successfully created index '{}':\n  Index path: {}\n  Files indexed: {}\n  Total content size: {}\n  Repository path: {}",
            stats.index_name,
            stats.index_path.display(),
            stats.file_count,
            format_size(stats.total_size),
            stats.repo_path.display()
        ))
    }
}

/// JSON formatter
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

#[derive(Serialize, Deserialize)]
struct SearchOutput {
    query: String,
    count: usize,
    results: Vec<SearchResultJson>,
}

#[derive(Serialize, Deserialize)]
struct SearchResultJson {
    title: String,
    path: String,
    score: f32,
    snippet: String,
}

impl ResultFormatter for JsonFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResult]) -> Result<String> {
        let output = SearchOutput {
            query: query.to_string(),
            count: results.len(),
            results: results
                .iter()
                .map(|r| SearchResultJson {
                    title: r.title.clone(),
                    path: r.path.clone(),
                    score: r.score,
                    snippet: r.snippet.clone(),
                })
                .collect(),
        };

        if self.pretty {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn format_index_list(&self, indexes: &[IndexInfo]) -> Result<String> {
        let output = serde_json::json!({
            "count": indexes.len(),
            "indexes": indexes.iter().map(|index| {
                serde_json::json!({
                    "name": index.name,
                    "path": index.path.display().to_string(),
                    "doc_count": index.metadata.doc_count,
                    "size_bytes": index.metadata.size_bytes,
                })
            }).collect::<Vec<_>>()
        });

        if self.pretty {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn format_indexing_stats(&self, stats: &IndexingStats) -> Result<String> {
        let output = serde_json::json!({
            "success": true,
            "index_name": stats.index_name,
            "index_path": stats.index_path.display().to_string(),
            "file_count": stats.file_count,
            "total_size": stats.total_size,
            "repo_path": stats.repo_path.display().to_string(),
        });

        if self.pretty {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }
}
