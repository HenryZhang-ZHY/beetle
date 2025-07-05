mod json;
mod text;
mod utils;

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;

use engine::search::SearchResultItem;

pub trait ResultFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String;
}
