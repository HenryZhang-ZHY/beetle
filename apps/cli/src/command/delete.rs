use super::{index_name, BeetleCommand};
use bpaf::*;

pub fn delete_command() -> OptionParser<BeetleCommand> {
    construct!(BeetleCommand::Delete { index_name() }).to_options()
}
