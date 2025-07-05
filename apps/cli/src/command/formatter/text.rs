use super::ResultFormatter;
use engine::search::SearchResultItem;

pub struct PlainTextFormatter;

impl ResultFormatter for PlainTextFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String {
        if results.is_empty() {
            return format!("No results found for query: '{}'", query);
        }

        let mut output = format!("Found {} results for query '{}':\n\n", results.len(), query);

        for result in results {
            output.push_str(&format!(
                "(Score: {:.2}) Path: {}\n   Snippet:\n{}\n\n",
                result.score, result.path, result.snippet
            ));
        }

        output
    }
}
