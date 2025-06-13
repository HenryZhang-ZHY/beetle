use std::env;

use beetle_engine::{
    list_indexes, new_index, search_index, IndexingOptions, JsonFormatter, PlainTextFormatter,
    QueryOptions,
};

use std::path::PathBuf;

use super::{BeetleCommand, CliRunResult, OutputFormat, Runner};

pub struct BeetleRunner {
    options: BeetleCommand,
}

impl BeetleRunner {
    fn get_beetle_home() -> String {
        let beetle_home = std::env::var("BEETLE_HOME").unwrap_or_else(|_| {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            format!("{}/.beetle", home_dir)
        });

        return beetle_home;
    }
}

impl Runner for BeetleRunner {
    type Options = BeetleCommand;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(self) -> CliRunResult {
        match self.options {
            BeetleCommand::New {
                index_name,
                path_to_be_indexed,
            } => {
                let beetle_home = BeetleRunner::get_beetle_home();
                let index_path = PathBuf::from(beetle_home).join("indexes").join(&index_name);

                match new_index(
                    &index_name,
                    &path_to_be_indexed,
                    &index_path,
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
            BeetleCommand::List => {
                let beetle_home = BeetleRunner::get_beetle_home();
                let index_path = PathBuf::from(beetle_home).join("indexes");

                let mut index_names = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&index_path) {
                    for entry in entries.flatten() {
                        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                            if let Some(name) = entry.file_name().to_str() {
                                index_names.push(name.to_string());
                            }
                        }
                    }
                }

                if index_names.is_empty() {
                    CliRunResult::PlainTextResult("No indexes found".to_string())
                } else {
                    let formatted_list = index_names.join("\n");
                    CliRunResult::PlainTextResult(formatted_list)
                }
            }

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
