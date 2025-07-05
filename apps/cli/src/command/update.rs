use super::{index_name, BeetleCommand};

use bpaf::*;

pub fn update_command() -> OptionParser<BeetleCommand> {
    let incremental = long("incremental")
        .switch()
        .help("Perform incremental update");

    let reindex = long("reindex").switch().help("Perform full reindex");

    construct!(BeetleCommand::Update {
        index_name(),
        incremental,
        reindex
    })
    .to_options()
}
