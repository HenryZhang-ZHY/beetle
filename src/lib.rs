use anyhow::{Context, Result};
use bpaf::*;
use std::fs;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, TantivyDocument};
use walkdir::WalkDir;

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

/// Create a new search index from a repository
fn create_index(index_name: &str, repo_path: &PathBuf, output_path: &PathBuf) -> Result<String> {
    // Create schema
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED);
    let path = schema_builder.add_text_field("path", STORED);
    let schema = schema_builder.build();

    // Create index directory
    let index_path = output_path.join(index_name);
    fs::create_dir_all(&index_path)
        .with_context(|| format!("Failed to create index directory: {}", index_path.display()))?;

    // Create index
    let index = Index::create_in_dir(&index_path, schema.clone())
        .with_context(|| "Failed to create tantivy index")?;

    let mut index_writer: IndexWriter = index
        .writer(50_000_000)
        .with_context(|| "Failed to create index writer")?;

    let mut file_count = 0;
    let mut total_size = 0u64;

    // Walk through repository and index files
    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path();

            // Skip binary files and common non-text files
            if let Some(extension) = file_path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if matches!(
                    ext.as_str(),
                    "exe"
                        | "dll"
                        | "so"
                        | "dylib"
                        | "bin"
                        | "obj"
                        | "o"
                        | "jpg"
                        | "jpeg"
                        | "png"
                        | "gif"
                        | "bmp"
                        | "ico"
                        | "mp3"
                        | "mp4"
                        | "avi"
                        | "mov"
                        | "wav"
                        | "zip"
                        | "tar"
                        | "gz"
                        | "rar"
                        | "7z"
                ) {
                    continue;
                }
            }

            // Try to read file content
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let relative_path = file_path
                        .strip_prefix(repo_path)
                        .unwrap_or(file_path)
                        .to_string_lossy();

                    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();

                    // Add document to index
                    index_writer.add_document(doc!(
                        title => file_name.as_ref(),
                        body => content.as_str(),
                        path => relative_path.as_ref(),
                    ))?;

                    file_count += 1;
                    total_size += content.len() as u64;

                    if file_count % 100 == 0 {
                        println!("Indexed {} files...", file_count);
                    }
                }
                Err(_) => {
                    // Skip files that can't be read as text
                    continue;
                }
            }
        }
    }

    // Commit the index
    index_writer
        .commit()
        .with_context(|| "Failed to commit index")?;

    Ok(format!(
        "Successfully created index '{}':\n  Index path: {}\n  Files indexed: {}\n  Total content size: {} bytes\n  Repository path: {}",
        index_name,
        index_path.display(),
        file_count,
        total_size,
        repo_path.display()
    ))
}

/// Search an existing index
fn search_index(index_name: &str, query_str: &str) -> Result<String> {
    // Try to find the index in common locations
    let possible_paths = vec![
        PathBuf::from(index_name),
        PathBuf::from("indexes").join(index_name),
        PathBuf::from("indices").join(index_name),
        PathBuf::from(".").join(index_name),
    ];

    let mut index_path = None;
    for path in possible_paths {
        if path.exists() && path.is_dir() {
            index_path = Some(path);
            break;
        }
    }

    let index_path = index_path.ok_or_else(|| {
        anyhow::anyhow!(
            "Index '{}' not found. Tried looking in current directory and common index locations.",
            index_name
        )
    })?;

    // Open the index
    let index = Index::open_in_dir(&index_path)
        .with_context(|| format!("Failed to open index at: {}", index_path.display()))?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .with_context(|| "Failed to create index reader")?;

    let searcher = reader.searcher();

    // Create query parser for title and body fields
    let schema = index.schema();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let path_field = schema.get_field("path").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    // Parse and execute query
    let query = query_parser
        .parse_query(query_str)
        .with_context(|| format!("Failed to parse query: '{}'", query_str))?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(10))
        .with_context(|| "Failed to execute search")?;

    if top_docs.is_empty() {
        return Ok(format!("No results found for query: '{}'", query_str));
    }

    let mut results = format!(
        "Found {} results for query '{}':\n\n",
        top_docs.len(),
        query_str
    );

    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher
            .doc(doc_address)
            .with_context(|| "Failed to retrieve document")?;

        let title_text = retrieved_doc
            .get_first(title)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let path_text = retrieved_doc
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown path");

        let body_text = retrieved_doc
            .get_first(body)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Extract relevant snippet from body
        let snippet = extract_snippet(body_text, query_str, 100);

        results.push_str(&format!(
            "📄 {} (score: {:.2})\n   Path: {}\n   Preview: {}\n\n",
            title_text, score, path_text, snippet
        ));
    }

    Ok(results)
}

/// List all available indexes
fn list_indexes() -> Result<String> {
    let search_paths = vec![
        PathBuf::from("indexes"),
        PathBuf::from("indices"),
        PathBuf::from("."),
    ];

    let mut found_indexes = Vec::new();

    for search_path in search_paths {
        if search_path.exists() && search_path.is_dir() {
            for entry in fs::read_dir(&search_path)
                .with_context(|| format!("Failed to read directory: {}", search_path.display()))?
            {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Check if it looks like a tantivy index (contains meta.json)
                    if path.join("meta.json").exists() {
                        let index_name = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // Get index info
                        let info = get_index_info(&path)?;
                        found_indexes.push((index_name, path, info));
                    }
                }
            }
        }
    }

    if found_indexes.is_empty() {
        return Ok("No indexes found. Create one with: beetle create <index_name> -p <repo_path> -o <output_path>".to_string());
    }

    let mut result = format!("Found {} index(es):\n\n", found_indexes.len());

    for (name, path, info) in found_indexes {
        result.push_str(&format!(
            "📂 {}\n   Path: {}\n   Documents: {}\n   Size: {}\n\n",
            name,
            path.display(),
            info.doc_count,
            format_size(info.size_bytes)
        ));
    }

    Ok(result)
}

struct IndexInfo {
    doc_count: u64,
    size_bytes: u64,
}

/// Get information about an index
fn get_index_info(index_path: &PathBuf) -> Result<IndexInfo> {
    // Calculate directory size
    let mut size_bytes = 0u64;
    if let Ok(entries) = fs::read_dir(index_path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size_bytes += metadata.len();
                }
            }
        }
    }

    // Try to open index and get document count
    let doc_count = match Index::open_in_dir(index_path) {
        Ok(index) => {
            match index
                .reader_builder()
                .reload_policy(ReloadPolicy::OnCommitWithDelay)
                .try_into()
            {
                Ok(reader) => {
                    let searcher = reader.searcher();
                    searcher.num_docs() as u64
                }
                Err(_) => 0,
            }
        }
        Err(_) => 0,
    };

    Ok(IndexInfo {
        doc_count,
        size_bytes,
    })
}

/// Format file size in human readable format
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
fn extract_snippet(text: &str, query: &str, max_length: usize) -> String {
    let query_words: Vec<&str> = query
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();

    if query_words.is_empty() || text.is_empty() {
        return if text.len() > max_length {
            format!("{}...", &text[..max_length])
        } else {
            text.to_string()
        };
    }

    // Find the first occurrence of any query word
    let text_lower = text.to_lowercase();
    let mut best_pos = None;
    let mut best_word_len = 0;

    for word in &query_words {
        let word_lower = word.to_lowercase();
        if let Some(pos) = text_lower.find(&word_lower) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
                best_word_len = word.len();
            }
        }
    }

    if let Some(pos) = best_pos {
        // Calculate snippet boundaries
        let start = if pos > 30 { pos - 30 } else { 0 };
        let end = std::cmp::min(text.len(), pos + best_word_len + 30);

        let mut snippet = text[start..end].to_string();

        // Clean up the snippet
        snippet = snippet.replace('\n', " ").replace('\t', " ");
        while snippet.contains("  ") {
            snippet = snippet.replace("  ", " ");
        }

        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };

        format!("{}{}{}", prefix, snippet.trim(), suffix)
    } else {
        // Fallback to beginning of text
        if text.len() > max_length {
            format!("{}...", &text[..max_length].replace('\n', " "))
        } else {
            text.replace('\n', " ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
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

    #[test]
    fn test_in_memory_index_creation() {
        // Test that we can create an in-memory index without file system side effects
        let index = create_memory_index();
        assert!(index.is_ok(), "Should be able to create in-memory index");
    }

    #[test]
    fn test_in_memory_index_with_documents() {
        // Test indexing and searching in memory
        let test_files = vec![
            (
                "main.rs",
                "src/main.rs",
                "fn main() { println!(\"Hello, world!\"); }",
            ),
            (
                "lib.rs",
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a + b }",
            ),
            (
                "test.rs",
                "tests/test.rs",
                "use mylib::add; #[test] fn test_add() { assert_eq!(add(2, 3), 5); }",
            ),
        ];

        let index = create_memory_index_with_documents(test_files);
        assert!(
            index.is_ok(),
            "Should be able to create index with documents"
        );

        if let Ok(index) = index {
            // Test searching the in-memory index
            let search_result = search_memory_index(&index, "main");
            assert!(search_result.is_ok(), "Should be able to search index");

            if let Ok(result) = search_result {
                assert!(
                    result.contains("main"),
                    "Search result should contain 'main'"
                );
            }
        }
    }

    #[test]
    fn test_mock_execute_command() {
        // Test that mock execute_command works without side effects
        let create_cmd = Command::Create {
            index_name: "test_mock".to_string(),
            repo_path: PathBuf::from("/nonexistent/path"),
            output_path: PathBuf::from("/nonexistent/output"),
        };

        let result = mock_execute_command(create_cmd);
        assert!(result.contains("Successfully created index"));
        assert!(result.contains("test_mock"));

        let search_cmd = Command::Search {
            index_name: "test_search".to_string(),
            query: "test query".to_string(),
        };

        let result = mock_execute_command(search_cmd);
        assert!(result.contains("Found 0 results"));
        assert!(result.contains("test query"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_extract_snippet() {
        let text = "This is a long piece of text that contains the word function somewhere in the middle and we want to extract a snippet around it.";
        let query = "function";
        let snippet = extract_snippet(text, query, 100);

        assert!(
            snippet.contains("function"),
            "Snippet should contain the query word"
        );
        assert!(snippet.len() <= 110, "Snippet should be reasonably sized"); // accounting for ellipsis
    }
}

#[cfg(test)]
mod test_utils {
    use super::*;
    use tantivy::directory::RamDirectory;

    /// Create an in-memory index for testing purposes
    pub fn create_memory_index() -> Result<Index> {
        let mut schema_builder = Schema::builder();
        let _title = schema_builder.add_text_field("title", TEXT | STORED);
        let _body = schema_builder.add_text_field("body", TEXT | STORED);
        let _path = schema_builder.add_text_field("path", STORED);
        let schema = schema_builder.build();

        let directory = RamDirectory::create();
        Index::create(directory, schema, Default::default())
            .with_context(|| "Failed to create in-memory tantivy index")
    }

    /// Create an in-memory index with sample documents for testing
    pub fn create_memory_index_with_documents(files: Vec<(&str, &str, &str)>) -> Result<Index> {
        let index = create_memory_index()?;
        let schema = index.schema();
        let title = schema.get_field("title").unwrap();
        let body = schema.get_field("body").unwrap();
        let path_field = schema.get_field("path").unwrap();

        let mut index_writer = index.writer(50_000_000)?;

        for (file_title, file_path, content) in files {
            index_writer.add_document(doc!(
                title => file_title,
                body => content,
                path_field => file_path,
            ))?;
        }

        index_writer.commit()?;
        Ok(index)
    }

    /// Mock execute_command function that doesn't create files
    pub fn mock_execute_command(command: Command) -> String {
        match command {
            Command::Create { index_name, repo_path, output_path } => {
                // Simulate successful creation without actually creating files
                format!(
                    "Successfully created index '{}':\n  Index path: {}\n  Files indexed: 0\n  Total content size: 0 bytes\n  Repository path: {}",
                    index_name,
                    output_path.join(&index_name).display(),
                    repo_path.display()
                )
            }
            Command::Search { index_name, query } => {
                // Simulate search results without actual index
                if index_name.is_empty() || query.is_empty() {
                    format!("No results found for query: '{}'", query)
                } else {
                    format!("Found 0 results for query '{}':\n\n", query)
                }
            }
            Command::List => {
                "No indexes found. Create one with: beetle create <index_name> -p <repo_path> -o <output_path>".to_string()
            }
        }
    }

    /// Search an in-memory index for testing
    pub fn search_memory_index(index: &Index, query_str: &str) -> Result<String> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .with_context(|| "Failed to create index reader")?;

        let searcher = reader.searcher();
        let schema = index.schema();
        let title = schema.get_field("title").unwrap();
        let body = schema.get_field("body").unwrap();
        let path_field = schema.get_field("path").unwrap();

        let query_parser = QueryParser::for_index(index, vec![title, body]);
        let query = query_parser
            .parse_query(query_str)
            .with_context(|| format!("Failed to parse query: '{}'", query_str))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(10))
            .with_context(|| "Failed to execute search")?;

        if top_docs.is_empty() {
            return Ok(format!("No results found for query: '{}'", query_str));
        }

        let mut results = format!(
            "Found {} results for query '{}':\n\n",
            top_docs.len(),
            query_str
        );

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher
                .doc(doc_address)
                .with_context(|| "Failed to retrieve document")?;

            let title_text = retrieved_doc
                .get_first(title)
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let path_text = retrieved_doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown path");

            results.push_str(&format!(
                "📄 {} (score: {:.2})\n   Path: {}\n\n",
                title_text, score, path_text
            ));
        }

        Ok(results)
    }
}
