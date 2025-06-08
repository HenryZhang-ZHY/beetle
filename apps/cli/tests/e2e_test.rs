use assert_cmd::Command;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// End-to-end tests for beetle CLI
///
/// Test Setup:
/// - BEETLE_HOME is set to a temporary directory
/// - Working directory is set to a temporary directory  
/// - Test fixtures are copied to the temporary working directory
///
/// Helper function to execute a beetle search command and return parsed JSON results
///
/// # Arguments
/// * `index_name` - Name of the beetle index to search
/// * `query` - Search query string
///
/// # Returns
/// * `Result<Value, Box<dyn std::error::Error>>` - Parsed JSON response or error
fn execute_search(index_name: &str, query: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let search_output = Command::cargo_bin("beetle")?
        .args(["search", "-i", index_name, "-q", query, "--format", "json"])
        .output()?;

    if !search_output.status.success() {
        eprintln!("beetle search for '{query}' failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&search_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&search_output.stderr));
        return Err(format!("beetle search command for '{query}' failed").into());
    }

    let search_stdout = String::from_utf8_lossy(&search_output.stdout);
    let json_str = search_stdout.trim();
    let json_result: Value = serde_json::from_str(json_str)?;

    Ok(json_result)
}

/// Helper function to validate search results
///
/// # Arguments
/// * `json_result` - Parsed JSON search results
/// * `expected_count` - Expected number of results
/// * `query` - Original search query for error messages
///
/// # Returns
/// * `Vec<&Value>` - Array of result objects
fn validate_search_results<'a>(
    json_result: &'a Value,
    expected_count: usize,
    query: &str,
) -> Vec<&'a Value> {
    let results = json_result["payload"]
        .as_array()
        .expect("JSON should contain 'payload' array");

    assert_eq!(
        results.len(),
        expected_count,
        "Search results for '{query}' should contain exactly {expected_count} result(s)"
    );

    results.iter().collect()
}

/// Helper function to find and validate a result by file path
///
/// # Arguments
/// * `results` - Array of search result objects
/// * `file_name` - File name to search for in paths
/// * `expected_snippet` - Text that should be present in the snippet
/// * `query` - Original search query for error messages
fn validate_result_by_file(
    results: &[&Value],
    file_name: &str,
    expected_snippet: &str,
    query: &str,
) {
    let result = results
        .iter()
        .find(|result| result["path"].as_str().unwrap_or("").contains(file_name))
        .unwrap_or_else(|| {
            panic!(
                "Should find {file_name} in search results for '{query}'"
            )
        });

    let snippet = result["snippet"].as_str().unwrap_or("");
    assert!(
        snippet.contains(expected_snippet),
        "Search results for '{query}' should contain '{expected_snippet}' in {file_name} snippet"
    );
}

/// User Story: C Programmer Onboarding
///
/// Scenario: A C programmer wants to index their project and search for code
///
/// Given: User has a C project with source files
/// When: User creates a new beetle index and searches for "int main"
/// Then: User sees relevant search results in the terminal
/// When: User searches for "a + b"
/// Then: User sees relevant search results in the terminal
/// When: User searches for "return"
/// Then: User sees relevant search results in the terminal
/// When: User deletes the main.c file
/// Then: User sees no search results for "int main"
#[test]
fn test_c_programmer_onboarding() {
    // Arrange: Set up test environment with temporary directories
    let beetle_home_dir = TempDir::new().expect("Failed to create temp dir for BEETLE_HOME");
    let working_dir = TempDir::new().expect("Failed to create temp dir for working directory");

    // Preserve original BEETLE_HOME for cleanup
    let original_beetle_home = env::var("BEETLE_HOME").ok();
    env::set_var("BEETLE_HOME", beetle_home_dir.path());

    // Copy test fixture (C project) to working directory
    let fixture_source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("c_project_add");

    let c_project_path = working_dir.path().join("c_project_add");
    copy_dir_all(&fixture_source, &c_project_path).expect("Failed to copy C project fixture");

    // Verify: C project fixture is properly set up
    assert!(c_project_path.exists(), "C project should exist");
    assert!(
        c_project_path.join("main.c").exists(),
        "main.c should exist"
    );
    assert!(c_project_path.join("add.h").exists(), "add.h should exist");

    // Act: Create new beetle index for the C project
    let new_output = Command::cargo_bin("beetle")
        .unwrap()
        .args([
            "new",
            "-i",
            "c_project_add",
            "-p",
            &c_project_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute beetle new command");

    // Assert: Index creation succeeded
    if !new_output.status.success() {
        eprintln!("beetle new failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&new_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&new_output.stderr));
        panic!("beetle new command failed");
    }

    // run update command to ensure index is up-to-date
    let _update_output = Command::cargo_bin("beetle")
        .unwrap()
        .args(["update", "-i", "c_project_add"])
        .output()
        .expect("Failed to execute beetle update command");

    // Test Case 1: Search for "int main"
    let json_result = execute_search("c_project_add", "\"int main\"")
        .expect("Search for 'int main' should succeed");

    let results = validate_search_results(&json_result, 1, "int main");
    validate_result_by_file(&results, "main.c", "int main", "int main");

    // Assert: Search results don't contain add.h as a separate file result
    let add_h_results: Vec<_> = results
        .iter()
        .filter(|result| result["path"].as_str().unwrap_or("").contains("add.h"))
        .collect();
    assert!(
        add_h_results.is_empty(),
        "Search results should not contain add.h as a separate file result"
    );

    // Test Case 2: Search for "a + b"
    let json_result =
        execute_search("c_project_add", "\"a + b\"").expect("Search for 'a + b' should succeed");

    let results = validate_search_results(&json_result, 1, "a + b");
    validate_result_by_file(&results, "add.h", "a + b", "a + b");

    // Test Case 3: Search for "return"
    let json_result =
        execute_search("c_project_add", "\"return\"").expect("Search for 'return' should succeed");

    let results = validate_search_results(&json_result, 2, "return");
    validate_result_by_file(&results, "main.c", "return", "return");
    validate_result_by_file(&results, "add.h", "return", "return");

    // Test Case 4: Delete main.c file and verify search results
    let main_c_path = c_project_path.join("main.c");
    fs::remove_file(&main_c_path).expect("Failed to delete main.c");

    // Run update command to refresh the index after file deletion
    let _update_output = Command::cargo_bin("beetle")
        .unwrap()
        .args(["update", "-i", "c_project_add"])
        .output()
        .expect("Failed to execute beetle update command after file deletion");

    // Search for "int main" should now return no results
    let json_result = execute_search("c_project_add", "\"main\"")
        .expect("Search for 'int main' should succeed even when no results");

    let results = validate_search_results(&json_result, 0, "main");
    assert!(
        results.is_empty(),
        "Search results for 'int main' should be empty after deleting main.c"
    );

    // Cleanup: Restore original BEETLE_HOME environment variable
    match original_beetle_home {
        Some(original) => env::set_var("BEETLE_HOME", original),
        None => env::remove_var("BEETLE_HOME"),
    }
}

/// Recursively copies a directory and all its contents to a destination path
///
/// # Arguments
/// * `src` - Source directory path to copy from
/// * `dst` - Destination directory path to copy to
///
/// # Returns
/// * `Ok(())` - If the copy operation succeeds
/// * `Err(std::io::Error)` - If any file system operation fails
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_type.is_dir() {
            // Recursively copy subdirectories
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            // Copy files directly
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
