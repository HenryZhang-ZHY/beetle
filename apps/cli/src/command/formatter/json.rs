use super::ResultFormatter;
use engine::search::SearchResultItem;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SearchOutput {
    query: String,
    count: usize,
    results: Vec<SearchResultJson>,
}

#[derive(Serialize, Deserialize)]
struct SearchResultJson {
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
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String {
        let output = SearchOutput {
            query: query.to_string(),
            count: results.len(),
            results: results
                .iter()
                .map(|r| SearchResultJson {
                    path: r.path.clone(),
                    score: r.score,
                    snippet: r.snippet.clone(),
                })
                .collect(),
        };

        if self.pretty {
            serde_json::to_string_pretty(&output).unwrap_or("".to_string())
        } else {
            serde_json::to_string(&output).unwrap_or("".to_string())
        }
    }
}
