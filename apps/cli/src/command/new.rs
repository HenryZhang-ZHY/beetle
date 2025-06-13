use super::{index_name, BeetleCommand};
use bpaf::*;
use std::path::PathBuf;

pub fn new() -> OptionParser<BeetleCommand> {
    let path = long("path")
        .short('p')
        .argument::<PathBuf>("PATH")
        .help("Path to the folder to be indexed");

    construct!(path, index_name())
        .map(|(repo_path, index_name)| BeetleCommand::New {
            index_name,
            path_to_be_indexed: repo_path,
        })
        .to_options()
}
