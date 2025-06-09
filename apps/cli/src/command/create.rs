use super::BeetleCommand;
use bpaf::*;
use std::path::PathBuf;

pub fn create_command() -> OptionParser<BeetleCommand> {
    let repo_path = long("path")
        .argument::<PathBuf>("PATH")
        .help("Path to the repository folder to be indexed");

    let index_name = positional::<String>("INDEX_NAME").help("Name of the index to create");

    construct!(repo_path, index_name)
        .map(|(repo_path, index_name)| BeetleCommand::Create {
            index_name,
            repo_path,
        })
        .to_options()
}
