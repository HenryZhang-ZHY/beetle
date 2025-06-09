mod result;

use beetle::cli::{beetle_command, BeetleRunner, CliRunResult, Runner};

fn main() -> CliRunResult {
    let command = beetle_command().run();

    BeetleRunner::new(command).run()
}
