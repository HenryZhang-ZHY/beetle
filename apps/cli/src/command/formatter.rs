mod json;
mod text;
mod utils;

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;
pub use utils::format_size;

use beetle_engine::{IndexInfo, IndexingStats, SearchResult};

pub trait ResultFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResult]) -> String;

    fn format_index_list(&self, indexes: &[IndexInfo]) -> String;

    fn format_indexing_stats(&self, stats: &IndexingStats) -> String;
}
