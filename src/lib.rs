use bpaf::*;
use std::path::PathBuf;

/// Command enum representing the different operations beetle can perform.
/// 
/// # Examples
/// 
/// Creating a new index:
/// ```
/// use beetle::Command;
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
/// use beetle::Command;
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
    
    let index_name = positional::<String>("INDEX_NAME")
        .help("Name of the index to create");

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
    
    let index_name = positional::<String>("INDEX_NAME")
        .help("Name of the index to search");

    construct!(Command::Search {
        query,
        index_name,
    })
    .to_options()
}

pub fn cli() -> OptionParser<Command> {
    let create = create_command().command("create")
        .help("Create a new search index");
    
    let search = search_command().command("search")
        .help("Search an existing index");

    construct!([create, search])
        .to_options()
        .descr("Beetle - A source code search tool")
        .header("Search and index source code repositories")
        .footer("Examples:\n  beetle create myindex -p /path/to/repo -o /path/to/index\n  beetle search myindex -q \"function name\"")
}

/// Execute a command and return the formatted output string.
/// 
/// This function takes a `Command` and returns a formatted string
/// describing what operation would be performed.
/// 
/// # Arguments
/// 
/// * `command` - The command to execute
/// 
/// # Examples
/// 
/// ```
/// use beetle::{Command, execute_command};
/// use std::path::PathBuf;
/// 
/// let cmd = Command::Create {
///     index_name: "test".to_string(),
///     repo_path: PathBuf::from("/repo"),
///     output_path: PathBuf::from("/output"),
/// };
/// 
/// let result = execute_command(cmd);
/// assert!(result.contains("Creating index"));
/// assert!(result.contains("Index name: test"));
/// ```
/// 
/// ```
/// use beetle::{Command, execute_command};
/// 
/// let cmd = Command::Search {
///     index_name: "test".to_string(),
///     query: "search term".to_string(),
/// };
/// 
/// let result = execute_command(cmd);
/// assert!(result.contains("Searching index"));
/// assert!(result.contains("Query: search term"));
/// ```
pub fn execute_command(command: Command) -> String {
    match command {
        Command::Create { index_name, repo_path, output_path } => {
            format!(
                "Creating index:\n  Index name: {}\n  Repository path: {}\n  Output path: {}",
                index_name,
                repo_path.display(),
                output_path.display()
            )
        }
        Command::Search { index_name, query } => {
            format!(
                "Searching index:\n  Index name: {}\n  Query: {}",
                index_name,
                query
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_create_command_execution() {
        let command = Command::Create {
            index_name: "test_index".to_string(),
            repo_path: PathBuf::from("/path/to/repo"),
            output_path: PathBuf::from("/path/to/output"),
        };

        let result = execute_command(command);
        
        assert!(result.contains("Creating index:"));
        assert!(result.contains("Index name: test_index"));
        assert!(result.contains("Repository path: /path/to/repo"));
        assert!(result.contains("Output path: /path/to/output"));
    }

    #[test]
    fn test_search_command_execution() {
        let command = Command::Search {
            index_name: "my_index".to_string(),
            query: "function main".to_string(),
        };

        let result = execute_command(command);
        
        assert!(result.contains("Searching index:"));
        assert!(result.contains("Index name: my_index"));
        assert!(result.contains("Query: function main"));
    }

    #[test]
    fn test_create_command_with_windows_paths() {
        let command = Command::Create {
            index_name: "windows_index".to_string(),
            repo_path: PathBuf::from(r"C:\Users\test\repo"),
            output_path: PathBuf::from(r"C:\Users\test\output"),
        };

        let result = execute_command(command);
        
        assert!(result.contains("Creating index:"));
        assert!(result.contains("Index name: windows_index"));
        // On Windows, paths should display with backslashes
        #[cfg(windows)]
        {
            assert!(result.contains(r"C:\Users\test\repo"));
            assert!(result.contains(r"C:\Users\test\output"));
        }
        // On Unix-like systems, paths might be displayed differently
        #[cfg(not(windows))]
        {
            assert!(result.contains("C:/Users/test/repo") || result.contains(r"C:\Users\test\repo"));
            assert!(result.contains("C:/Users/test/output") || result.contains(r"C:\Users\test\output"));
        }
    }

    #[test]
    fn test_search_command_with_special_characters() {
        let command = Command::Search {
            index_name: "special_index".to_string(),
            query: "struct MyStruct { field: String }".to_string(),
        };

        let result = execute_command(command);
        
        assert!(result.contains("Searching index:"));
        assert!(result.contains("Index name: special_index"));
        assert!(result.contains("Query: struct MyStruct { field: String }"));
    }

    #[test]
    fn test_command_clone() {
        let original_command = Command::Create {
            index_name: "clone_test".to_string(),
            repo_path: PathBuf::from("/test/path"),
            output_path: PathBuf::from("/test/output"),
        };

        let cloned_command = original_command.clone();
        
        // Both commands should produce the same output
        let original_result = execute_command(original_command);
        let cloned_result = execute_command(cloned_command);
        
        assert_eq!(original_result, cloned_result);
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

        let result = execute_command(command);
        
        assert!(result.contains("Searching index:"));
        assert!(result.contains("Index name: "));
        assert!(result.contains("Query: "));
    }

    #[test]
    fn test_long_strings() {
        let long_name = "a".repeat(1000);
        let long_query = "b".repeat(2000);
        
        let command = Command::Search {
            index_name: long_name.clone(),
            query: long_query.clone(),
        };

        let result = execute_command(command);
        
        assert!(result.contains(&long_name));
        assert!(result.contains(&long_query));
    }
}
