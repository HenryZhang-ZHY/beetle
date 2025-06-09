use super::{IndexInfo, IndexingStats, QueryResult, Result, ResultFormatter};

use serde::{Deserialize, Serialize};

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

/// JSON formatter
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl ResultFormatter for JsonFormatter {
    fn format_search_results(&self, query: &str, results: &[QueryResult]) -> Result<String> {
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
