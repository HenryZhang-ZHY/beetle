use std::process::{ExitCode, Termination};

#[derive(Debug)]
#[allow(dead_code)]
pub enum CliRunResult {
    None,
    Success(String),
    Error(String),
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::SUCCESS,
            Self::Success(text) => {
                println!("{}", text);
                ExitCode::SUCCESS
            }
            Self::Error(err_text) => {
                eprintln!("Error: {}", err_text);
                ExitCode::FAILURE
            }
        }
    }
}
