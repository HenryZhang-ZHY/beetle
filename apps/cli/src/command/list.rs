use super::{format, BeetleCommand};
use bpaf::*;

pub fn list_command() -> OptionParser<BeetleCommand> {
    construct!(BeetleCommand::List {
        format()
    })
    .to_options()
}
