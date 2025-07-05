use std::process::{ExitCode, Termination};

#[derive(Debug)]
#[allow(dead_code)]
pub enum CliRunResult {
    None,
    PlainTextResult(String),
}

impl Termination for CliRunResult {
    fn report(self) -> ExitCode {
        match self {
            Self::None => ExitCode::SUCCESS,
            Self::PlainTextResult(text) => {
                println!("{}", text);
                ExitCode::SUCCESS
            }
        }
    }
}
