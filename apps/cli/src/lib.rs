use beetle_engine::{create_index, list_indexes, search_index};
use bpaf::*;
use std::path::PathBuf;

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

    let index_name = positional::<String>("INDEX_NAME").help("Name of the index to search");

    construct!(Command::Search { query, index_name }).to_options()
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
        .footer("Examples:\n  beetle create myindex -p /path/to/repo -o /path/to/index\n  beetle search myindex -q \"function name\"\n  beetle list")
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
        } => match create_index(&index_name, &repo_path, &output_path) {
            Ok(message) => message,
            Err(e) => format!("Error creating index: {}", e),
        },
        Command::Search { index_name, query } => match search_index(&index_name, &query) {
            Ok(results) => results,
            Err(e) => format!("Error searching index: {}", e),
        },
        Command::List => match list_indexes() {
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
        };

        // Test that command is created correctly
        match command {
            Command::Search { index_name, query } => {
                assert_eq!(index_name, "my_index");
                assert_eq!(query, "function main");
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
        };

        // Test that empty strings are handled
        match command {
            Command::Search { index_name, query } => {
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
        };

        match command {
            Command::Search { index_name, query } => {
                assert_eq!(index_name, long_name);
                assert_eq!(query, long_query);
            }
            _ => panic!("Expected Search command"),
        }
    }
}
