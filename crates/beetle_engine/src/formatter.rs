mod json;
mod text;

use anyhow::Result;

use crate::index_manager::{IndexInfo, IndexingStats};
use crate::query::QueryResult;
use crate::utils::format_size;

/// Trait for formatting various results
pub trait ResultFormatter {
    /// Format search results
    fn format_search_results(&self, query: &str, results: &[QueryResult]) -> Result<String>;

    /// Format index list
    fn format_index_list(&self, indexes: &[IndexInfo]) -> Result<String>;

    /// Format indexing stats
    fn format_indexing_stats(&self, stats: &IndexingStats) -> Result<String>;
}

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;
