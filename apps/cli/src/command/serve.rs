use super::BeetleCommand;
use bpaf::*;

pub fn serve_command() -> OptionParser<BeetleCommand> {
    let port = long("port")
        .short('p')
        .help("Port to bind the server to")
        .argument("PORT")
        .fallback(3000);

    construct!(port)
        .map(|port| BeetleCommand::Serve { port })
        .to_options()
}
