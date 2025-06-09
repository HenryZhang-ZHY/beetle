use super::{BeetleCommand, OutputFormat};
use bpaf::*;

pub fn query_command() -> OptionParser<BeetleCommand> {
    let search = long("search")
        .argument::<String>("QUERY")
        .help("Search query");

    let index_name = long("index")
        .argument::<String>("INDEX_NAME")
        .help("Name of the index to query");

    let formatter = long("format")
        .argument::<String>("FORMAT")
        .help("Output format: text (default) or json")
        .parse(|s| match s.as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Invalid format. Use 'text' or 'json'"),
        })
        .fallback(OutputFormat::Text);

    construct!(BeetleCommand::Query {
        index_name,
        search,
        formatter
    })
    .to_options()
}
