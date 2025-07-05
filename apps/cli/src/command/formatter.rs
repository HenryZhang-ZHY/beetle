mod json;
mod text;
mod utils;

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;
pub use utils::format_size;

use engine::SearchResultItem;

pub trait ResultFormatter {
    fn format_search_results(&self, query: &str, results: &[SearchResultItem]) -> String;
}
