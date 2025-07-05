use super::{index_name, BeetleCommand};

use bpaf::*;

pub fn update_command() -> OptionParser<BeetleCommand> {
    let reindex = long("reindex").switch().help("Perform full reindex");

    construct!(BeetleCommand::Update {
        index_name(),
        reindex
    })
    .to_options()
}
