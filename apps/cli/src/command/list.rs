use super::BeetleCommand;
use bpaf::*;

pub fn list_command() -> OptionParser<BeetleCommand> {
    pure(BeetleCommand::List).to_options()
}
