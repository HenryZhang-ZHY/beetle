use engine::storage::FsStorage;
use engine::IndexCatalog;

use std::path::PathBuf;

use super::{BeetleCommand, JsonFormatter, OutputFormat, PlainTextFormatter, ResultFormatter};
use crate::cli::{get_beetle_home, CliRunResult, Runner};
// use crate::server::HttpServer;

pub struct BeetleRunner {
    options: BeetleCommand,
    catalog: IndexCatalog,
}

impl Runner for BeetleRunner {
    type Options = BeetleCommand;

    fn new(options: Self::Options) -> Self {
        let storage = FsStorage::new(PathBuf::from(get_beetle_home()));
        let catalog = IndexCatalog::new(storage);

        Self { options, catalog }
    }

    fn run(self) -> CliRunResult {
        match self.options {
            BeetleCommand::New {
                index_name,
                path_to_be_indexed,
            } => match self
                .catalog
                .create(&index_name, &path_to_be_indexed.to_string_lossy())
            {
                Ok(_) => CliRunResult::PlainTextResult(format!(
                    "Index '{}' created successfully",
                    index_name
                )),
                Err(e) => CliRunResult::PlainTextResult(format!("{}", e)),
            },
            BeetleCommand::Search {
                index_name,
                query,
                formatter,
            } => {
                let mut searcher = self.catalog.get_searcher(&index_name).unwrap();
                let search_result = searcher.search(&query).unwrap();
                match formatter {
                    OutputFormat::Text => {
                        let text_formatter = PlainTextFormatter;

                        CliRunResult::PlainTextResult(
                            text_formatter.format_search_results(&query, &search_result),
                        )
                    }
                    OutputFormat::Json => {
                        let json_formatter = JsonFormatter::new(true);

                        CliRunResult::PlainTextResult(
                            json_formatter.format_search_results(&query, &search_result),
                        )
                    }
                }
            }
            BeetleCommand::List => match self.catalog.list() {
                Ok(indexes) => {
                    if indexes.is_empty() {
                        return CliRunResult::PlainTextResult("No indexes found".to_string());
                    }

                    let plain_text_result = indexes
                        .iter()
                        .map(|metadata| {
                            format!(
                                "Index Name: {}, Target Path: {}",
                                metadata.index_name, metadata.target_path
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    CliRunResult::PlainTextResult(plain_text_result)
                }
                Err(e) => CliRunResult::PlainTextResult(format!("Error listing indexes: {}", e)),
            },

            BeetleCommand::Remove { index_name } => match self.catalog.remove(&index_name) {
                Ok(_) => CliRunResult::PlainTextResult(format!(
                    "Index '{}' removed successfully",
                    index_name
                )),
                Err(e) => CliRunResult::PlainTextResult(format!("{}", e)),
            },
            BeetleCommand::Update {
                index_name,
                reindex,
            } => {
                let mut writer = self.catalog.get_writer(&index_name).unwrap();

                if reindex {
                    self.catalog.reset(&index_name).unwrap();
                }

                match writer.index() {
                    Ok(_) => CliRunResult::PlainTextResult(format!(
                        "Incremental update for '{}' successful",
                        index_name
                    )),
                    Err(e) => CliRunResult::PlainTextResult(format!(
                        "Failed to index data for '{}': {}",
                        index_name, e
                    )),
                }
            }
            BeetleCommand::Serve { port } => CliRunResult::None,
        }
    }
}
