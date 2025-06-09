use super::BeetleCommand;
use bpaf::*;

pub fn delete_command() -> OptionParser<BeetleCommand> {
    let index_name = long("index")
        .argument::<String>("INDEX_NAME")
        .help("Name of the index to delete");

    construct!(BeetleCommand::Delete { index_name }).to_options()
}
