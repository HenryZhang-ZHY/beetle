use engine::storage::FsStorage;
use engine::IndexCatalog;

use tracing::trace;

use std::path::PathBuf;

use super::{BeetleCommand, JsonFormatter, OutputFormat, PlainTextFormatter, ResultFormatter};
use crate::{
    cli::{get_beetle_home, CliRunResult, Runner},
    command::formatter::CommandOutput,
    server::HttpServer,
};

pub struct BeetleRunner {
    options: BeetleCommand,
    catalog: IndexCatalog,
}

impl BeetleRunner {
    fn execute(self) -> Result<CommandOutput, String> {
        match self.options {
            BeetleCommand::New {
                index_name,
                path_to_be_indexed,
            } => {
                self.catalog
                    .create(&index_name, &path_to_be_indexed.to_string_lossy())?;

                Ok(CommandOutput::Success(format!(
                    "Index '{index_name}' created successfully"
                )))
            }
            BeetleCommand::Search {
                index_name, query, ..
            } => {
                let searcher = self.catalog.get_searcher(&index_name)?;
                let search_result = searcher.search(&query)?;

                Ok(CommandOutput::Search(search_result))
            }
            BeetleCommand::List { .. } => {
                let indexes = self.catalog.list()?;

                Ok(CommandOutput::List(indexes))
            }
            BeetleCommand::Remove { index_name } => {
                self.catalog.remove(&index_name)?;

                Ok(CommandOutput::Success(format!(
                    "Index '{index_name}' removed successfully"
                )))
            }
            BeetleCommand::Update {
                index_name,
                reindex,
            } => {
                let mut writer = self.catalog.get_writer(&index_name)?;

                if reindex {
                    self.catalog.reset(&index_name)?;
                }

                writer.index()?;

                Ok(CommandOutput::Success(format!(
                    "Incremental update for '{index_name}' successful"
                )))
            }
            BeetleCommand::Serve { port } => Ok(HttpServer::start(port)),
        }
    }
}

impl Runner for BeetleRunner {
    type Options = BeetleCommand;

    fn new(options: Self::Options) -> Self {
        let storage = FsStorage::new(PathBuf::from(get_beetle_home()));
        let catalog = IndexCatalog::new(storage);

        Self { options, catalog }
    }

    fn run(self) -> CliRunResult {
        let output_format = match &self.options {
            BeetleCommand::Search { format, .. } => format.clone(),
            BeetleCommand::List { format } => format.clone(),
            _ => OutputFormat::Text,
        };

        trace!("output format: {:?}", output_format);

        match self.execute() {
            Ok(output) => {
                let formatted_string = match output_format {
                    OutputFormat::Json => JsonFormatter::new(true).format(output),
                    OutputFormat::Text => PlainTextFormatter.format(output),
                };
                CliRunResult::Success(formatted_string)
            }
            Err(message) => CliRunResult::Error(message),
        }
    }
}
