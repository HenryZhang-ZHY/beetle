use super::{index_name, BeetleCommand, OutputFormat};
use bpaf::*;

pub fn search_command() -> OptionParser<BeetleCommand> {
    let query = long("query")
        .short('q')
        .argument::<String>("QUERY_EXPRESSION")
        .help("Search query expression");

    let formatter = long("format")
        .argument::<String>("FORMAT")
        .help("Output format: text (default) or json")
        .parse(|s| match s.as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Invalid format. Use 'text' or 'json'"),
        })
        .fallback(OutputFormat::Text);

    construct!(BeetleCommand::Search {
        index_name(),
        query,
        formatter
    })
    .to_options()
}
