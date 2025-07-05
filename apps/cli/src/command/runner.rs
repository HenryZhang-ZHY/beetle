use axum::extract::path;
use engine::{new_index, FsStorage, IndexCatalog, IndexManager, IndexingOptions};

use std::path::PathBuf;

use super::{BeetleCommand, JsonFormatter, OutputFormat, PlainTextFormatter, ResultFormatter};
use crate::cli::{get_beetle_home, CliRunResult, Runner};
use crate::server::HttpServer;

pub struct BeetleRunner {
    options: BeetleCommand,
    catalog: IndexCatalog,
}

impl BeetleRunner {
    fn get_index_path(index_name: &str) -> PathBuf {
        let beetle_home = get_beetle_home();
        PathBuf::from(beetle_home).join("indexes").join(index_name)
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
        match self.options {
            BeetleCommand::New {
                index_name,
                path_to_be_indexed,
            } => match self
                .catalog
                .create(&index_name, &path_to_be_indexed.to_string_lossy())
            {
                Ok(_) => CliRunResult::PlainTextResult(format!(
                    "Index '{}' created successfully at '{}'",
                    index_name,
                    BeetleRunner::get_index_path(&index_name).display()
                )),
                Err(e) => CliRunResult::PlainTextResult(format!("{}", e)),
            },
            BeetleCommand::Search {
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
                let beetle_home = get_beetle_home();
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

            BeetleCommand::Remove { index_name } => match self.catalog.remove(&index_name) {
                Ok(_) => CliRunResult::PlainTextResult(format!(
                    "Index '{}' removed successfully",
                    index_name
                )),
                Err(e) => CliRunResult::PlainTextResult(format!("{}", e)),
            },
            BeetleCommand::Update {
                index_name,
                incremental,
                reindex,
            } => {
                // TODO: Implement update_index in engine
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
                    CliRunResult::PlainTextResult(
                        "Please specify either --incremental or --reindex for update".to_string(),
                    )
                }
            }
            BeetleCommand::Serve { port } => HttpServer::start(port),
        }
    }
}
