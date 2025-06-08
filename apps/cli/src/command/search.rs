use super::{format, index_name, BeetleCommand};
use bpaf::*;

pub fn search_command() -> OptionParser<BeetleCommand> {
    let query = long("query")
        .short('q')
        .argument::<String>("QUERY_EXPRESSION")
        .help("Search query expression");

    construct!(BeetleCommand::Search {
        index_name(),
        query,
        format()
    })
    .to_options()
}
