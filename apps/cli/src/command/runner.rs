use beetle_engine::{new_index, IndexManager, IndexingOptions};

use std::path::PathBuf;

use crate::command::new;

use super::{
    BeetleCommand, CliRunResult, JsonFormatter, OutputFormat, PlainTextFormatter, ResultFormatter,
    Runner,
};

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

    fn get_index_path(index_name: &str) -> PathBuf {
        let beetle_home = Self::get_beetle_home();
        PathBuf::from(beetle_home).join("indexes").join(index_name)
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
                let index_path: PathBuf = BeetleRunner::get_index_path(&index_name);

                match new_index(
                    &index_name,
                    &path_to_be_indexed,
                    &index_path,
                    IndexingOptions::new(),
                ) {
                    Ok(stats) => CliRunResult::PlainTextResult(
                        PlainTextFormatter.format_indexing_stats(&stats),
                    ),
                    Err(e) => CliRunResult::PlainTextResult(format!("Error creating index: {}", e)),
                }
            }
            BeetleCommand::Query {
                index_name,
                query,
                formatter,
            } => {
                let index_path: PathBuf = BeetleRunner::get_index_path(&index_name);
                let index_manager = IndexManager::new(index_path);

                let search_result = index_manager.search(&query);

                match formatter {
                    OutputFormat::Text => {
                        let results = search_result.unwrap();
                        let text_formatter = PlainTextFormatter;

                        CliRunResult::PlainTextResult(
                            text_formatter.format_search_results(&query, &results),
                        )
                    }
                    OutputFormat::Json => {
                        let results = search_result.unwrap();
                        let json_formatter = JsonFormatter::new(true);

                        CliRunResult::PlainTextResult(
                            json_formatter.format_search_results(&query, &results),
                        )
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
