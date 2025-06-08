mod result;

use beetle::cli::{beetle_command, BeetleRunner, CliRunResult, Runner};

fn main() -> CliRunResult {
    init_tracing();

    let command = beetle_command().run();

    BeetleRunner::new(command).run()
}

/// `BEETLE_LOG=trace beetle list`
fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("BEETLE_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
