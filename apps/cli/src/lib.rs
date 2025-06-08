mod command;
mod result;
mod runner;
mod server;
pub mod static_files;

pub mod cli {
    use std::path::PathBuf;

    pub use crate::{
        command::{beetle_command, BeetleRunner, CommandOutput},
        result::CliRunResult,
        runner::Runner,
        server::HttpServer,
    };

    pub fn get_beetle_home() -> String {
        std::env::var("BEETLE_HOME").unwrap_or_else(|_| {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());

            PathBuf::from(home_dir)
                .join(".beetle")
                .to_string_lossy()
                .into_owned()
        })
    }
}
