use super::BeetleCommand;
use bpaf::*;

pub fn update_command() -> OptionParser<BeetleCommand> {
    let index_name = long("index")
        .argument::<String>("INDEX_NAME")
        .help("Name of the index to update");

    let incremental = long("incremental")
        .switch()
        .help("Perform incremental update");

    let reindex = long("reindex").switch().help("Perform full reindex");

    construct!(BeetleCommand::Update {
        index_name,
        incremental,
        reindex
    })
    .to_options()
}
