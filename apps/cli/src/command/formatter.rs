mod json;
mod text;
mod utils;

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;
pub use utils::format_size;

use engine::{IndexingStats, SearchResultItem};

pub trait ResultFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String;

    fn format_indexing_stats(&self, stats: &IndexingStats) -> String;
}
