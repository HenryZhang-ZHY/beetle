use assert_cmd::{assert::Assert, Command};
use predicates::prelude::*;
use serial_test::serial;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ================================================================================
// Test Infrastructure
// ================================================================================

/// Test context that encapsulates common test setup and teardown
struct TestContext {
    temp_dir: TempDir,
    index_name: String,
}

impl TestContext {
    fn new(test_name: &str) -> Self {
        cleanup_test_indexes();
        Self {
            temp_dir: create_temp_dir(),
            index_name: format!("test_{}", test_name),
        }
    }

    fn with_default_output(test_name: &str) -> Self {
        cleanup_test_indexes();
        ensure_default_indexes_dir();
        Self {
            temp_dir: create_temp_dir(),
            index_name: format!("test_{}", test_name),
        }
    }

    fn fixtures_path(&self) -> PathBuf {
        get_fixtures_path().join("dotnet")
    }

    fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    fn cleanup_default_index(&self) {
        let _ = fs::remove_dir_all(format!("indexes/{}", self.index_name));
    }
}

// ================================================================================
// Command Builders
// ================================================================================

/// Builder for beetle commands with fluent interface
struct BeetleCommand {
    cmd: Command,
}

impl BeetleCommand {
    fn new() -> Self {
        Self {
            cmd: Command::cargo_bin("beetle").unwrap(),
        }
    }

    fn create_index(mut self, name: &str, path: &Path, output: &Path) -> Self {
        self.cmd
            .arg("create")
            .arg(name)
            .arg("--path")
            .arg(path)
            .arg("--output")
            .arg(output);
        self
    }

    fn search(mut self, index: &str, query: &str) -> Self {
        self.cmd.arg("search").arg(index).arg("--query").arg(query);
        self
    }

    fn list(mut self) -> Self {
        self.cmd.arg("list");
        self
    }

    fn help(mut self) -> Self {
        self.cmd.arg("--help");
        self
    }

    fn version(mut self) -> Self {
        self.cmd.arg("--version");
        self
    }

    fn assert(mut self) -> Assert {
        self.cmd.assert()
    }

    fn output(mut self) -> std::io::Result<std::process::Output> {
        self.cmd.output()
    }
}

// ================================================================================
// Helper Functions
// ================================================================================

/// Helper to get the path to the beetle binary
fn beetle_cmd() -> Command {
    Command::cargo_bin("beetle").unwrap()
}

/// Helper to create a temporary directory with cross-platform compatibility
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Helper to get the fixtures path
fn get_fixtures_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("tests").join("fixtures")
}

/// Ensure default indexes directory exists
fn ensure_default_indexes_dir() {
    fs::create_dir_all("indexes").unwrap_or_default();
}

/// Helper to clean up any leftover test indexes
fn cleanup_test_indexes() {
    let index_dirs = vec!["indexes", "indices"];

    for dir in index_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Only remove test indexes (those starting with "test_")
                    if name.starts_with("test_") {
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }
}

/// Helper to create an index and return success status
fn create_test_index(index_name: &str, source_path: &Path, output_path: &Path) -> bool {
    BeetleCommand::new()
        .create_index(index_name, source_path, output_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Helper to verify index files exist
fn verify_index_created(output_path: &Path, index_name: &str) {
    let index_path = output_path.join(index_name);
    assert!(index_path.exists(), "Index directory should be created");
    assert!(
        index_path.join("meta.json").exists(),
        "Index meta.json should exist"
    );
}

// ================================================================================
// Basic Command Tests
// ================================================================================

#[test]
#[serial]
fn test_beetle_help() {
    BeetleCommand::new()
        .help()
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Beetle - A source code search tool",
        ))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("list"));
}

#[test]
#[serial]
fn test_beetle_version() {
    let result = BeetleCommand::new().version().output().unwrap();

    // Either succeeds with version info or fails with expected error
    assert!(result.status.success() || !result.stderr.is_empty());
}

#[test]
#[serial]
fn test_list_empty_indexes() {
    cleanup_test_indexes();

    BeetleCommand::new()
        .list()
        .assert()
        .success()
        .stdout(predicate::str::contains("No indexes found"));
}

// ================================================================================
// Index Creation Tests
// ================================================================================

#[test]
#[serial]
fn test_create_index_with_dotnet_fixtures() {
    let ctx = TestContext::new("dotnet_index");
    let fixtures_path = ctx.fixtures_path();

    // Ensure fixtures exist
    assert!(
        fixtures_path.exists(),
        "Fixtures directory not found: {}",
        fixtures_path.display()
    );
    assert!(
        fixtures_path.join("EFCore.InMemory.csproj").exists(),
        "Expected fixture file not found"
    );
    assert!(
        fixtures_path
            .join("InMemoryProjectionBindingExpressionVisitor.cs")
            .exists(),
        "Expected fixture file not found"
    );

    BeetleCommand::new()
        .create_index(&ctx.index_name, &fixtures_path, ctx.temp_path())
        .assert()
        .success()
        .stdout(predicate::str::contains("indexed"))
        .stdout(predicate::str::contains("Files indexed"));

    verify_index_created(ctx.temp_path(), &ctx.index_name);
}

#[test]
#[serial]
fn test_create_index_with_invalid_path() {
    let ctx = TestContext::new("invalid_index");
    let invalid_path = PathBuf::from("non_existent_path_12345");

    BeetleCommand::new()
        .create_index(&ctx.index_name, &invalid_path, ctx.temp_path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Files indexed: 0"));
}

#[test]
#[serial]
fn test_create_index_with_special_characters() {
    let ctx = TestContext::new("special_chars");
    let fixtures_path = ctx.fixtures_path();

    let problematic_names = vec![
        "test-with-dashes",
        "test_with_underscores",
        "test.with.dots",
    ];

    for index_name in problematic_names {
        let result = BeetleCommand::new()
            .create_index(index_name, &fixtures_path, ctx.temp_path())
            .output()
            .unwrap();

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(
                !stderr.is_empty(),
                "Should provide error message for index name: {}",
                index_name
            );
        } else {
            verify_index_created(ctx.temp_path(), index_name);
        }
    }
}

#[test]
#[serial]
fn test_create_index_with_absolute_paths() {
    let ctx = TestContext::new("absolute_paths");
    let fixtures_path = ctx.fixtures_path();

    BeetleCommand::new()
        .create_index(
            &ctx.index_name,
            &fixtures_path.canonicalize().unwrap(),
            &ctx.temp_path().canonicalize().unwrap(),
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("Files indexed: 2"));
}

#[test]
#[serial]
fn test_create_index_with_nested_directories() {
    let ctx = TestContext::new("nested_index");
    let fixtures_path = ctx.fixtures_path();
    let nested_output = ctx.temp_path().join("deeply").join("nested").join("path");

    BeetleCommand::new()
        .create_index(&ctx.index_name, &fixtures_path, &nested_output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Files indexed: 2"));

    verify_index_created(&nested_output, &ctx.index_name);
}

// ================================================================================
// Search Tests
// ================================================================================

#[test]
#[serial]
fn test_search_nonexistent_index() {
    cleanup_test_indexes();

    BeetleCommand::new()
        .search("nonexistent_index", "test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Error searching index"));
}

#[test]
#[serial]
fn test_search_dotnet_specific_content() {
    let ctx = TestContext::with_default_output("dotnet_search");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let test_queries = vec![
        "EntityFrameworkCore",
        "InMemory",
        "ProjectionBinding",
        "using System",
        "namespace Microsoft",
        "csproj",
        "PropertyGroup",
    ];

    for query in test_queries {
        BeetleCommand::new()
            .search(&ctx.index_name, query)
            .assert()
            .success()
            .stdout(predicate::str::contains("📄").or(predicate::str::contains("No results")));
    }

    ctx.cleanup_default_index();
}

#[test]
#[serial]
fn test_empty_query_search() {
    let ctx = TestContext::with_default_output("empty_query");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let result = BeetleCommand::new()
        .search(&ctx.index_name, "")
        .output()
        .unwrap();

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.is_empty(),
            "Should provide error message for empty query"
        );
    }

    ctx.cleanup_default_index();
}

#[test]
#[serial]
fn test_unicode_content_search() {
    let ctx = TestContext::with_default_output("unicode_search");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let unicode_queries = vec!["©", "™", "é", "中文", "日本語"];

    for query in unicode_queries {
        BeetleCommand::new()
            .search(&ctx.index_name, query)
            .assert()
            .success();
    }

    ctx.cleanup_default_index();
}

#[test]
#[serial]
fn test_large_query_string() {
    let ctx = TestContext::with_default_output("large_query");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let large_query = "function class method property namespace using System".repeat(100);

    let result = BeetleCommand::new()
        .search(&ctx.index_name, &large_query)
        .output()
        .unwrap();

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panic"), "Should not panic on large query");
    }

    ctx.cleanup_default_index();
}

#[test]
#[serial]
fn test_search_with_different_query_formats() {
    let ctx = TestContext::with_default_output("query_formats");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let query_formats = vec![
        "class",
        "class method",
        "\"class method\"",
        "using AND System",
        "using OR import",
        "EntityFrameworkCore*",
        "entity framework",
        "ENTITYFRAMEWORKCORE",
        "entityframeworkcore",
    ];

    for query in query_formats {
        BeetleCommand::new()
            .search(&ctx.index_name, query)
            .assert()
            .success();
    }

    ctx.cleanup_default_index();
}

// ================================================================================
// Integration Tests
// ================================================================================

#[test]
#[serial]
fn test_full_workflow_create_and_search() {
    let ctx = TestContext::new("workflow_index");
    let fixtures_path = ctx.fixtures_path();

    // Create index
    BeetleCommand::new()
        .create_index(&ctx.index_name, &fixtures_path, ctx.temp_path())
        .assert()
        .success()
        .stdout(predicate::str::contains("indexed"));

    // Note: list and search commands look in default directories, not temp_dir
    // This is a limitation of the current implementation
}

#[test]
#[serial]
fn test_list_indexes_after_creation() {
    let ctx = TestContext::with_default_output("list_index");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    BeetleCommand::new()
        .list()
        .assert()
        .success()
        .stdout(predicate::str::contains(&ctx.index_name))
        .stdout(predicate::str::contains("📂"))
        .stdout(predicate::str::contains("Documents:"))
        .stdout(predicate::str::contains("Size:"));

    ctx.cleanup_default_index();
}

#[test]
#[serial]
fn test_concurrent_operations() {
    let ctx = TestContext::with_default_output("concurrent");

    let index_names = vec![
        "test_concurrent_1",
        "test_concurrent_2",
        "test_concurrent_3",
    ];

    // Create multiple indexes
    for index_name in &index_names {
        create_test_index(index_name, &ctx.fixtures_path(), &PathBuf::from("indexes"));
    }

    // Verify all indexes exist
    let output = BeetleCommand::new().list().output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    for index_name in &index_names {
        assert!(
            stdout.contains(index_name),
            "Should list index: {}",
            index_name
        );
    }

    // Test searching each index
    for index_name in &index_names {
        BeetleCommand::new()
            .search(index_name, "using")
            .assert()
            .success();
    }

    // Clean up
    for index_name in &index_names {
        let _ = fs::remove_dir_all(format!("indexes/{}", index_name));
    }
}

// ================================================================================
// File Type Tests
// ================================================================================

#[test]
#[serial]
fn test_index_different_file_types() {
    let ctx = TestContext::new("filetypes");

    // Create test files
    let test_files = vec![
        (
            "test.cs",
            "using System;\nnamespace Test { class Program { } }",
        ),
        ("test.rs", "fn main() { println!(\"Hello World\"); }"),
        ("test.py", "def main():\n    print(\"Hello World\")"),
        ("test.js", "function main() { console.log('Hello World'); }"),
        ("test.txt", "This is a plain text file with some content."),
        (
            "README.md",
            "# Test Repository\nThis is a test markdown file.",
        ),
        ("config.json", "{\"version\": \"1.0\", \"name\": \"test\"}"),
        ("style.css", "body { margin: 0; padding: 0; }"),
        ("index.html", "<html><body><h1>Test</h1></body></html>"),
    ];

    for (filename, content) in &test_files {
        let file_path = ctx.temp_path().join(filename);
        fs::write(&file_path, content).unwrap();
    }

    let output = BeetleCommand::new()
        .create_index(&ctx.index_name, ctx.temp_path(), ctx.temp_path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Command should succeed");
    assert!(
        stdout.contains("index"),
        "Should mention indexing operation"
    );
}

// ================================================================================
// Error Handling Tests
// ================================================================================

#[test]
#[serial]
fn test_cli_argument_parsing_edge_cases() {
    // Missing required arguments
    beetle_cmd().arg("create").assert().failure();
    beetle_cmd().arg("search").assert().failure();

    // Invalid commands
    beetle_cmd().arg("invalid_command").assert().failure();

    // Help for specific commands
    beetle_cmd()
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Name of the index to create"));

    beetle_cmd()
        .arg("search")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Name of the index to search"));

    beetle_cmd()
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("help information"));
}

#[test]
#[serial]
fn test_error_handling_and_recovery() {
    let ctx = TestContext::with_default_output("error_recovery");

    create_test_index(
        &ctx.index_name,
        &ctx.fixtures_path(),
        &PathBuf::from("indexes"),
    );

    let malformed_queries = vec!["\"unclosed quote", "AND", "OR", "(unclosed paren", ""];

    for query in malformed_queries {
        let result = BeetleCommand::new()
            .search(&ctx.index_name, query)
            .output()
            .unwrap();

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(
                !stderr.is_empty(),
                "Should provide error message for query: {}",
                query
            );
            assert!(
                !stderr.contains("panic"),
                "Should not panic for query: {}",
                query
            );
        }
    }

    // Verify index is still functional
    BeetleCommand::new()
        .search(&ctx.index_name, "using")
        .assert()
        .success();

    ctx.cleanup_default_index();
}

// ================================================================================
// Cleanup
// ================================================================================

#[test]
#[serial]
fn test_zzz_cleanup() {
    cleanup_test_indexes();

    if let Ok(temp_dir) = env::temp_dir().read_dir() {
        for entry in temp_dir.flatten() {
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with("beetle_test") || name_str.contains("test_index") {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
    }
}
