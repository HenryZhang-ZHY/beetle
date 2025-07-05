mod formatter;
mod list;
mod new;
mod option;
mod remove;
mod runner;
mod search;
mod serve;
mod update;

pub use runner::BeetleRunner;

pub use formatter::{JsonFormatter, PlainTextFormatter, ResultFormatter};

pub use option::index_name;

use bpaf::*;
use std::path::PathBuf;

use list::list_command;
use new::new_command;
use remove::remove_command;
use search::search_command;
use serve::serve_command;
use update::update_command;

/// Output format for search results
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Plain text format (default)
    Text,
    /// JSON format
    Json,
}

#[derive(Debug, Clone)]
pub enum BeetleCommand {
    /// Create a new search index from a repository
    New {
        index_name: String,
        /// Path to the repository folder to be indexed
        path_to_be_indexed: PathBuf,
    },
    /// Query an existing index
    Search {
        index_name: String,
        query: String,
        formatter: OutputFormat,
    },
    List,
    /// Delete an existing index
    Remove {
        /// Name of the index to remove
        index_name: String,
    },
    /// Update an existing index
    Update {
        /// Name of the index to update
        index_name: String,
        /// Whether to perform incremental update
        incremental: bool,
        /// Whether to perform full reindex
        reindex: bool,
    },
    /// Start HTTP server
    Serve {
        /// Port to bind the server to
        port: u16,
    },
}

pub fn beetle_command() -> OptionParser<BeetleCommand> {
    let new = new_command()
        .command("new")
        .help("Create a new index for a specified folder");

    let search = search_command()
        .command("search")
        .help("Search within an existing index");

    let list = list_command()
        .command("list")
        .help("Display all available indexes");

    let remove = remove_command()
        .command("remove")
        .help("Remove an index from the system");

    let update = update_command()
        .command("update")
        .help("Update an existing index with new changes or reindex");

    let serve = serve_command()
        .command("serve")
        .help("Start HTTP server for search API");

    construct!([new, search, list, remove, update, serve])
        .to_options()
        .descr("Beetle - Source Code Repository Indexing Tool")
        .header("Efficiently index and query source code repositories")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command_parsing() {
        let args = Args::from(&["new", "-i", "my-index", "--path", "/path/to/repo"]);
        let parser = beetle_command();

        let result = parser.run_inner(args);

        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::New {
                index_name,
                path_to_be_indexed: repo_path,
            } => {
                assert_eq!(index_name, "my-index");
                assert_eq!(repo_path, PathBuf::from("/path/to/repo"));
            }
            _ => panic!("Expected Create command"),
        }

        // Test missing path argument
        let args = Args::from(&["new", "my-index"]);
        let result = parser.run_inner(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_command_parsing() {
        let parser = beetle_command();

        // Test query with default text format
        let args = Args::from(&["search", "--index", "my-index", "--query", "main function"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Search {
                index_name,
                query,
                formatter,
            } => {
                assert_eq!(index_name, "my-index");
                assert_eq!(query, "main function");
                matches!(formatter, OutputFormat::Text);
            }
            _ => panic!("Expected Query command"),
        }

        // Test query with JSON format
        let args = Args::from(&[
            "search", "--index", "test-idx", "--query", "TODO", "--format", "json",
        ]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Search { formatter, .. } => {
                matches!(formatter, OutputFormat::Json);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_list_command_parsing() {
        let parser = beetle_command();

        let args = Args::from(&["list"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::List => {}
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_remove_command_parsing() {
        let parser = beetle_command();

        let args = Args::from(&["remove", "--index", "old-index"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Remove { index_name } => {
                assert_eq!(index_name, "old-index");
            }
            _ => panic!("Expected Delete command"),
        }

        // Test missing index argument
        let args = Args::from(&["remove"]);
        let result = parser.run_inner(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_command_parsing() {
        let parser = beetle_command();

        // Test incremental update
        let args = Args::from(&["update", "--index", "my-index", "--incremental"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Update {
                index_name,
                incremental,
                reindex,
            } => {
                assert_eq!(index_name, "my-index");
                assert!(incremental);
                assert!(!reindex);
            }
            _ => panic!("Expected Update command"),
        }

        // Test full reindex
        let args = Args::from(&["update", "--index", "my-index", "--reindex"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Update {
                index_name,
                incremental,
                reindex,
            } => {
                assert_eq!(index_name, "my-index");
                assert!(!incremental);
                assert!(reindex);
            }
            _ => panic!("Expected Update command"),
        }

        // Test both flags
        let args = Args::from(&[
            "update",
            "--index",
            "my-index",
            "--incremental",
            "--reindex",
        ]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Update {
                incremental,
                reindex,
                ..
            } => {
                assert!(incremental);
                assert!(reindex);
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_invalid_commands() {
        let parser = beetle_command();

        // Test unknown command
        let args = Args::from(&["unknown"]);
        let result = parser.run_inner(args);
        assert!(result.is_err());

        // Test no command
        let args = Args::from(&[]);
        let result = parser.run_inner(args);
        assert!(result.is_err());

        // Test invalid arguments
        let args = Args::from(&["new", "--invalid-flag"]);
        let result = parser.run_inner(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let parser = beetle_command();

        // Test help for main command
        let args = Args::from(&["--help"]);
        let result = parser.run_inner(args);
        assert!(result.is_err()); // Help returns an error with help message

        // Test help for subcommands
        let args = Args::from(&["new", "--help"]);
        let result = parser.run_inner(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_cases() {
        let parser = beetle_command(); // Test empty strings
        let args = Args::from(&["search", "--index", "", "--query", ""]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Search {
                index_name, query, ..
            } => {
                assert_eq!(index_name, "");
                assert_eq!(query, "");
            }
            _ => panic!("Expected Query command"),
        }

        // Test special characters in arguments
        let args = Args::from(&[
            "new",
            "--index",
            "index-with-dashes",
            "--path",
            "/path/with spaces/and-dashes",
        ]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        // Test Unicode in search query
        let args = Args::from(&["search", "--index", "test", "--query", "你好 world 🦀"]);
        let result = parser.run_inner(args);
        assert!(result.is_ok());

        match result.unwrap() {
            BeetleCommand::Search { query, .. } => {
                assert_eq!(query, "你好 world 🦀");
            }
            _ => panic!("Expected Query command"),
        }
    }
}
