use beetle_engine::{
    create_index, list_indexes, search_index, IndexingOptions, JsonFormatter, PlainTextFormatter,
    SearchOptions,
};
use bpaf::*;
use std::path::PathBuf;

/// Output format for search results
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Plain text format (default)
    Text,
    /// JSON format
    Json,
}

/// Command enum representing the different operations beetle can perform.
///
/// # Examples
///
/// Creating a new index:
/// ```
/// use beetle_cli::Command;
/// use std::path::PathBuf;
///
/// let cmd = Command::Create {
///     index_name: "my_index".to_string(),
///     repo_path: PathBuf::from("/path/to/repo"),
///     output_path: PathBuf::from("/path/to/output"),
/// };
/// ```
///
/// Searching an existing index:
/// ```
/// use beetle_cli::Command;
///
/// let cmd = Command::Search {
///     index_name: "my_index".to_string(),
///     query: "function main".to_string(),
///     formatter: beetle_cli::OutputFormat::Text,
/// };
/// ```
#[derive(Debug, Clone)]
pub enum Command {
    /// Create a new search index from a repository
    Create {
        /// Name of the index to create
        index_name: String,
        /// Path to the repository folder to be indexed
        repo_path: PathBuf,
        /// Path where the index files will be stored
        output_path: PathBuf,
    },
    /// Search an existing index
    Search {
        /// Name of the index to search
        index_name: String,
        /// Search query string
        query: String,
        /// Output format for results
        formatter: OutputFormat,
    },
    /// List all available indexes
    List,
}

pub fn create_command() -> OptionParser<Command> {
    let repo_path = short('p')
        .long("path")
        .argument::<PathBuf>("PATH")
        .help("Path to the repository folder to be indexed");

    let output_path = short('o')
        .long("output")
        .argument::<PathBuf>("OUTPUT")
        .help("Path for the index files");

    let index_name = positional::<String>("INDEX_NAME").help("Name of the index to create");

    construct!(Command::Create {
        repo_path,
        output_path,
        index_name,
    })
    .to_options()
}

pub fn search_command() -> OptionParser<Command> {
    let query = short('q')
        .long("query")
        .argument::<String>("QUERY")
        .help("Search query");

    let formatter = short('f')
        .long("formatter")
        .argument::<String>("FORMAT")
        .help("Output format: text (default) or json")
        .parse(|s| match s.as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Invalid format. Use 'text' or 'json'"),
        })
        .fallback(OutputFormat::Text);

    let index_name = positional::<String>("INDEX_NAME").help("Name of the index to search");

    construct!(Command::Search {
        query,
        formatter,
        index_name
    })
    .to_options()
}

pub fn list_command() -> OptionParser<Command> {
    pure(Command::List).to_options()
}

pub fn cli() -> OptionParser<Command> {
    let create = create_command()
        .command("create")
        .help("Create a new search index");

    let search = search_command()
        .command("search")
        .help("Search an existing index");

    let list = list_command()
        .command("list")
        .help("List all available indexes");

    construct!([create, search, list])
        .to_options()
        .descr("Beetle - A source code search tool")
        .header("Search and index source code repositories")
        .footer("Examples:\n  beetle create myindex -p /path/to/repo -o /path/to/index\n  beetle search myindex -q \"function name\"\n  beetle search myindex -q \"function name\" --formatter json\n  beetle list")
}

/// Execute a command and return the formatted output string.
///
/// This function takes a `Command` and executes the actual indexing or searching operation.
///
/// # Arguments
///
/// * `command` - The command to execute
///
/// # Examples
///
/// ```
/// use beetle_cli::{Command, execute_command};
/// use std::path::PathBuf;
///
/// let cmd = Command::Create {
///     index_name: "test".to_string(),
///     repo_path: PathBuf::from("/repo"),
///     output_path: PathBuf::from("/output"),
/// };
///
/// let result = execute_command(cmd);
/// // This will actually create the index
/// ```
pub fn execute_command(command: Command) -> String {
    match command {
        Command::Create {
            index_name,
            repo_path,
            output_path,
        } => match create_index(
            &index_name,
            &repo_path,
            &output_path,
            IndexingOptions::new(),
            &PlainTextFormatter,
        ) {
            Ok(message) => message,
            Err(e) => format!("Error creating index: {}", e),
        },
        Command::Search {
            index_name,
            query,
            formatter,
        } => {
            match formatter {
                OutputFormat::Text => {
                    match search_index(
                        &index_name,
                        &query,
                        SearchOptions::default(),
                        &PlainTextFormatter,
                    ) {
                        Ok(results) => results,
                        Err(e) => format!("Error searching index: {}", e),
                    }
                }
                OutputFormat::Json => {
                    match search_index(
                        &index_name,
                        &query,
                        SearchOptions::default(),
                        &JsonFormatter::new(true), // Use pretty JSON
                    ) {
                        Ok(results) => results,
                        Err(e) => format!("Error searching index: {}", e),
                    }
                }
            }
        }
        Command::List => match list_indexes(&PlainTextFormatter) {
            Ok(list) => list,
            Err(e) => format!("Error listing indexes: {}", e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_command_creation() {
        let command = Command::Create {
            index_name: "test_index".to_string(),
            repo_path: PathBuf::from("/path/to/repo"),
            output_path: PathBuf::from("/path/to/output"),
        };

        // Test that command is created correctly
        match command {
            Command::Create {
                index_name,
                repo_path,
                output_path,
            } => {
                assert_eq!(index_name, "test_index");
                assert_eq!(repo_path, PathBuf::from("/path/to/repo"));
                assert_eq!(output_path, PathBuf::from("/path/to/output"));
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_search_command_creation() {
        let command = Command::Search {
            index_name: "my_index".to_string(),
            query: "function main".to_string(),
            formatter: OutputFormat::Text,
        };

        // Test that command is created correctly
        match command {
            Command::Search {
                index_name,
                query,
                formatter,
            } => {
                assert_eq!(index_name, "my_index");
                assert_eq!(query, "function main");
                matches!(formatter, OutputFormat::Text);
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_command_clone() {
        let original_command = Command::Create {
            index_name: "clone_test".to_string(),
            repo_path: PathBuf::from("/test/path"),
            output_path: PathBuf::from("/test/output"),
        };

        let cloned_command = original_command.clone();

        // Both commands should be identical
        match (original_command, cloned_command) {
            (
                Command::Create {
                    index_name: n1,
                    repo_path: r1,
                    output_path: o1,
                },
                Command::Create {
                    index_name: n2,
                    repo_path: r2,
                    output_path: o2,
                },
            ) => {
                assert_eq!(n1, n2);
                assert_eq!(r1, r2);
                assert_eq!(o1, o2);
            }
            _ => panic!("Commands should be identical"),
        }
    }

    #[test]
    fn test_command_debug() {
        let command = Command::Search {
            index_name: "debug_test".to_string(),
            query: "test query".to_string(),
            formatter: OutputFormat::Json,
        };

        let debug_output = format!("{:?}", command);

        assert!(debug_output.contains("Search"));
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("test query"));
    }

    #[test]
    fn test_empty_strings() {
        let command = Command::Search {
            index_name: "".to_string(),
            query: "".to_string(),
            formatter: OutputFormat::Text,
        };

        // Test that empty strings are handled
        match command {
            Command::Search {
                index_name,
                query,
                formatter: _,
            } => {
                assert_eq!(index_name, "");
                assert_eq!(query, "");
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_long_strings() {
        let long_name = "a".repeat(1000);
        let long_query = "b".repeat(2000);

        let command = Command::Search {
            index_name: long_name.clone(),
            query: long_query.clone(),
            formatter: OutputFormat::Json,
        };

        match command {
            Command::Search {
                index_name,
                query,
                formatter: _,
            } => {
                assert_eq!(index_name, long_name);
                assert_eq!(query, long_query);
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_formatter_options() {
        // Test text formatter
        let text_command = Command::Search {
            index_name: "test".to_string(),
            query: "test".to_string(),
            formatter: OutputFormat::Text,
        };

        match text_command {
            Command::Search { formatter, .. } => {
                matches!(formatter, OutputFormat::Text);
            }
            _ => panic!("Expected Search command"),
        }

        // Test JSON formatter
        let json_command = Command::Search {
            index_name: "test".to_string(),
            query: "test".to_string(),
            formatter: OutputFormat::Json,
        };

        match json_command {
            Command::Search { formatter, .. } => {
                matches!(formatter, OutputFormat::Json);
            }
            _ => panic!("Expected Search command"),
        }
    }
}
