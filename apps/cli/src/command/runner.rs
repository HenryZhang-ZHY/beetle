use std::env;

use beetle_engine::{
    create_index, list_indexes, search_index, IndexingOptions, JsonFormatter, PlainTextFormatter,
    QueryOptions,
};

use std::path::PathBuf;

use super::{BeetleCommand, CliRunResult, OutputFormat, Runner};

pub struct BeetleRunner {
    options: BeetleCommand,
    cwd: PathBuf,
}

impl Runner for BeetleRunner {
    type Options = BeetleCommand;

    fn new(options: Self::Options) -> Self {
        Self {
            options,
            cwd: env::current_dir().expect("Failed to get current working directory"),
        }
    }

    fn run(self) -> CliRunResult {
        match self.options {
            BeetleCommand::Create {
                index_name,
                repo_path,
            } => {
                // Use ~/.beetle/indexes/<index_name> as the output path by default
                let beetle_home = std::env::var("BEETLE_HOME").unwrap_or_else(|_| {
                    let home_dir = std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .unwrap_or_else(|_| ".".to_string());
                    format!("{}/.beetle", home_dir)
                });
                let output_path = PathBuf::from(beetle_home).join("indexes").join(&index_name);

                match create_index(
                    &index_name,
                    &repo_path,
                    &output_path,
                    IndexingOptions::new(),
                    &PlainTextFormatter,
                ) {
                    Ok(message) => CliRunResult::PlainTextResult(message),
                    Err(e) => CliRunResult::PlainTextResult(format!("Error creating index: {}", e)),
                }
            }
            BeetleCommand::Query {
                index_name,
                search,
                formatter,
            } => {
                match formatter {
                    OutputFormat::Text => {
                        match search_index(
                            &index_name,
                            &search,
                            QueryOptions::default(),
                            &PlainTextFormatter,
                        ) {
                            Ok(results) => CliRunResult::PlainTextResult(results),
                            Err(e) => CliRunResult::PlainTextResult(format!(
                                "Error querying index: {}",
                                e
                            )),
                        }
                    }
                    OutputFormat::Json => {
                        match search_index(
                            &index_name,
                            &search,
                            QueryOptions::default(),
                            &JsonFormatter::new(true), // Use pretty JSON
                        ) {
                            Ok(results) => CliRunResult::PlainTextResult(results),
                            Err(e) => CliRunResult::PlainTextResult(format!(
                                "Error querying index: {}",
                                e
                            )),
                        }
                    }
                }
            }
            BeetleCommand::List => match list_indexes(&PlainTextFormatter) {
                Ok(list) => CliRunResult::PlainTextResult(list),
                Err(e) => CliRunResult::PlainTextResult(format!("Error listing indexes: {}", e)),
            },
            BeetleCommand::Delete { index_name } => {
                // TODO: Implement delete_index in beetle_engine
                CliRunResult::PlainTextResult(format!(
                    "Deleting index '{}' is not yet implemented",
                    index_name
                ))
            }
            BeetleCommand::Update {
                index_name,
                incremental,
                reindex,
            } => {
                // TODO: Implement update_index in beetle_engine
                if incremental {
                    CliRunResult::PlainTextResult(format!(
                        "Incremental update of index '{}' is not yet implemented",
                        index_name
                    ))
                } else if reindex {
                    CliRunResult::PlainTextResult(format!(
                        "Reindexing '{}' is not yet implemented",
                        index_name
                    ))
                } else {
                    CliRunResult::PlainTextResult(format!(
                        "Please specify either --incremental or --reindex for update"
                    ))
                }
            }
        }
    }
}
