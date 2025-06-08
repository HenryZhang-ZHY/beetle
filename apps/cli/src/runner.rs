use crate::result::CliRunResult;

pub trait Runner {
    type Options;

    fn new(matches: Self::Options) -> Self;

    fn run(self) -> CliRunResult;
}
