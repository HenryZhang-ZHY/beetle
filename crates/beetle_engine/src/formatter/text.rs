use super::{format_size, IndexInfo, IndexingStats, QueryResult, Result, ResultFormatter};

/// Plain text formatter (human-readable)
pub struct PlainTextFormatter;

impl ResultFormatter for PlainTextFormatter {
    fn format_search_results(&self, query: &str, results: &[QueryResult]) -> Result<String> {
        if results.is_empty() {
            return Ok(format!("No results found for query: '{}'", query));
        }

        let mut output = format!("Found {} results for query '{}':\n\n", results.len(), query);

        for result in results {
            output.push_str(&format!(
                "📄 (score: {:.2}) Path: {}\n   Preview: {}\n\n",
                result.score, result.path, result.snippet
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
