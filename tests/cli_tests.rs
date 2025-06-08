use beetle::Command;
use std::path::PathBuf;

#[test]
fn test_create_command_parsing() {
    // Test the command structure directly since we can't easily mock CLI args
    let command = Command::Create {
        index_name: "my_index".to_string(),
        repo_path: PathBuf::from("/path/to/repo"),
        output_path: PathBuf::from("/path/to/output"),
    };

    match command {
        Command::Create { index_name, repo_path, output_path } => {
            assert_eq!(index_name, "my_index");
            assert_eq!(repo_path, PathBuf::from("/path/to/repo"));
            assert_eq!(output_path, PathBuf::from("/path/to/output"));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_search_command_parsing() {
    let command = Command::Search {
        index_name: "test_index".to_string(),
        query: "function main".to_string(),
    };

    match command {
        Command::Search { index_name, query } => {
            assert_eq!(index_name, "test_index");
            assert_eq!(query, "function main");
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_command_variants() {
    // Test that both command variants can be created
    let create_cmd = Command::Create {
        index_name: "create_test".to_string(),
        repo_path: PathBuf::from("/create/path"),
        output_path: PathBuf::from("/create/output"),
    };

    let search_cmd = Command::Search {
        index_name: "search_test".to_string(),
        query: "search query".to_string(),
    };

    // Verify they're different variants
    match (&create_cmd, &search_cmd) {
        (Command::Create { .. }, Command::Search { .. }) => {
            // This is expected
        }
        _ => panic!("Commands should be different variants"),
    }
}

#[test]
fn test_pathbuf_handling() {
    let windows_path = PathBuf::from(r"C:\Users\test\repo");
    let unix_path = PathBuf::from("/home/user/repo");
    let relative_path = PathBuf::from("./relative/path");

    let cmd1 = Command::Create {
        index_name: "path_test1".to_string(),
        repo_path: windows_path.clone(),
        output_path: unix_path.clone(),
    };

    let cmd2 = Command::Create {
        index_name: "path_test2".to_string(),
        repo_path: relative_path.clone(),
        output_path: PathBuf::from("."),
    };

    match cmd1 {
        Command::Create { repo_path, output_path, .. } => {
            assert_eq!(repo_path, windows_path);
            assert_eq!(output_path, unix_path);
        }
        _ => panic!("Expected Create command"),
    }

    match cmd2 {
        Command::Create { repo_path, output_path, .. } => {
            assert_eq!(repo_path, relative_path);
            assert_eq!(output_path, PathBuf::from("."));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_string_handling() {
    // Test various string edge cases
    let empty_string = String::new();
    let unicode_string = "测试中文".to_string();
    let special_chars = "!@#$%^&*()_+{}|:<>?[]\\;'\".,/".to_string();
    let multiline_string = "line1\nline2\nline3".to_string();

    let cmd = Command::Search {
        index_name: unicode_string.clone(),
        query: special_chars.clone(),
    };

    match cmd {
        Command::Search { index_name, query } => {
            assert_eq!(index_name, unicode_string);
            assert_eq!(query, special_chars);
        }
        _ => panic!("Expected Search command"),
    }

    // Test empty strings
    let empty_cmd = Command::Search {
        index_name: empty_string.clone(),
        query: multiline_string.clone(),
    };

    match empty_cmd {
        Command::Search { index_name, query } => {
            assert_eq!(index_name, empty_string);
            assert_eq!(query, multiline_string);
        }
        _ => panic!("Expected Search command"),
    }
}
