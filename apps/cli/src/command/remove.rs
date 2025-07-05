use super::{index_name, BeetleCommand};
use bpaf::*;

pub fn remove_command() -> OptionParser<BeetleCommand> {
    construct!(BeetleCommand::Remove { index_name() }).to_options()
}
