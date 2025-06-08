mod json;
mod text;

pub use json::JsonFormatter;
pub use text::PlainTextFormatter;

use engine::search::SearchResultItem;

use engine::storage::IndexStorageMetadata;

pub enum CommandOutput {
    Search(Vec<SearchResultItem>),
    List(Vec<IndexStorageMetadata>),
    Success(String),
    Error(String),
}

pub trait ResultFormatter {
    fn format(&self, output: CommandOutput) -> String;
}
