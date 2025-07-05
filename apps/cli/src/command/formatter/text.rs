use super::{format_size, IndexingStats, ResultFormatter};
use engine::SearchResultItem;

pub struct PlainTextFormatter;

impl ResultFormatter for PlainTextFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String {
        if results.is_empty() {
            return format!("No results found for query: '{}'", query);
        }

        let mut output = format!("Found {} results for query '{}':\n\n", results.len(), query);

        for result in results {
            output.push_str(&format!(
                "📄 (score: {:.2}) Path: {}\n   Preview: {}\n\n",
                result.score, result.path, result.snippet
            ));
        }

        output
    }

    fn format_indexing_stats(&self, stats: &IndexingStats) -> String {
        format!(
            "Successfully created index '{}':\n  Index path: {}\n  Files indexed: {}\n  Total content size: {}\n  Repository path: {}",
            stats.index_name,
            stats.index_path.display(),
            stats.file_count,
            format_size(stats.total_size),
            stats.repo_path.display()
        )
    }
}
