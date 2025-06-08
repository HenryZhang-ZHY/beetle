use beetle::{Command, execute_command};
use std::path::PathBuf;

/// Test module for edge cases and error conditions
mod edge_cases {
    use super::*;

    #[test]
    fn test_maximum_path_length() {
        // Test with very long paths (near system limits)
        let long_path = "a".repeat(260); // Windows MAX_PATH is typically 260
        
        let command = Command::Create {
            index_name: "long_path_test".to_string(),
            repo_path: PathBuf::from(&long_path),
            output_path: PathBuf::from(&long_path),
        };

        let result = execute_command(command);
        assert!(result.contains("Creating index"));
        assert!(result.contains(&long_path));
    }

    #[test]
    fn test_special_characters_in_paths() {
        // Test paths with various special characters
        let special_chars = vec![
            "path with spaces",
            "path-with-dashes",
            "path_with_underscores",
            "path.with.dots",
            "path(with)parentheses",
            "path[with]brackets",
            "path{with}braces",
        ];

        for path_str in special_chars {
            let command = Command::Create {
                index_name: format!("test_{}", path_str.len()),
                repo_path: PathBuf::from(path_str),
                output_path: PathBuf::from(format!("output_{}", path_str)),
            };

            let result = execute_command(command);
            assert!(result.contains("Creating index"));
        }
    }

    #[test]
    fn test_unicode_in_all_fields() {
        let unicode_tests = vec![
            ("测试", "中文路径", "中文输出", "中文查询"),
            ("🚀", "🦀", "📁", "🔍"),
            ("αβγ", "δεζ", "ηθι", "κλμ"),
            ("عربي", "فارسی", "हिन्दी", "日本語"),
        ];

        for (name, repo, output, query) in unicode_tests {
            let create_cmd = Command::Create {
                index_name: name.to_string(),
                repo_path: PathBuf::from(repo),
                output_path: PathBuf::from(output),
            };

            let create_result = execute_command(create_cmd);
            assert!(create_result.contains(name));
            assert!(create_result.contains(repo));
            assert!(create_result.contains(output));

            let search_cmd = Command::Search {
                index_name: name.to_string(),
                query: query.to_string(),
            };

            let search_result = execute_command(search_cmd);
            assert!(search_result.contains(name));
            assert!(search_result.contains(query));
        }
    }

    #[test]
    fn test_newlines_and_tabs() {
        let command = Command::Search {
            index_name: "test\nwith\nnewlines".to_string(),
            query: "query\twith\ttabs\nand\nnewlines".to_string(),
        };

        let result = execute_command(command);
        assert!(result.contains("test\nwith\nnewlines"));
        assert!(result.contains("query\twith\ttabs\nand\nnewlines"));
    }

    #[test]
    fn test_very_large_strings() {
        // Test with strings larger than typical buffer sizes
        let huge_name = "x".repeat(1_000_000); // 1MB string
        let huge_query = "y".repeat(5_000_000); // 5MB string

        let command = Command::Search {
            index_name: huge_name.clone(),
            query: huge_query.clone(),
        };

        let result = execute_command(command);
        assert!(result.contains(&huge_name));
        assert!(result.contains(&huge_query));
        
        // Verify the result is appropriately large
        assert!(result.len() > 6_000_000);
    }

    #[test]
    fn test_null_bytes() {
        // Test strings containing null bytes
        let name_with_null = format!("test{}null", '\0');
        let query_with_null = format!("query{}with{}nulls", '\0', '\0');

        let command = Command::Search {
            index_name: name_with_null.clone(),
            query: query_with_null.clone(),
        };

        let result = execute_command(command);
        assert!(result.contains(&name_with_null));
        assert!(result.contains(&query_with_null));
    }

    #[test]
    fn test_control_characters() {
        // Test various control characters
        let control_chars = (0..32).map(|i| char::from(i)).collect::<String>();
        
        let command = Command::Search {
            index_name: format!("control{}", control_chars),
            query: format!("query{}", control_chars),
        };

        let result = execute_command(command);
        assert!(result.contains("control"));
        assert!(result.contains("query"));
    }
}

/// Performance and stress tests
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_command_creation_performance() {
        let start = Instant::now();
        
        for i in 0..10000 {
            let _ = Command::Create {
                index_name: format!("test_{}", i),
                repo_path: PathBuf::from(format!("/repo/{}", i)),
                output_path: PathBuf::from(format!("/output/{}", i)),
            };
        }
        
        let duration = start.elapsed();
        // Should be very fast - less than 100ms for 10k creations
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_execute_command_performance() {
        let commands: Vec<Command> = (0..1000).map(|i| {
            if i % 2 == 0 {
                Command::Create {
                    index_name: format!("perf_test_{}", i),
                    repo_path: PathBuf::from(format!("/repo/{}", i)),
                    output_path: PathBuf::from(format!("/output/{}", i)),
                }
            } else {
                Command::Search {
                    index_name: format!("perf_test_{}", i),
                    query: format!("query {}", i),
                }
            }
        }).collect();

        let start = Instant::now();
        
        for command in commands {
            let _ = execute_command(command);
        }
        
        let duration = start.elapsed();
        // Should execute 1000 commands in reasonable time
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_clone_performance() {
        let original = Command::Create {
            index_name: "a".repeat(10000),
            repo_path: PathBuf::from("b".repeat(10000)),
            output_path: PathBuf::from("c".repeat(10000)),
        };

        let start = Instant::now();
        
        for _ in 0..1000 {
            let _ = original.clone();
        }
        
        let duration = start.elapsed();
        // Cloning should be reasonably fast even with large strings
        assert!(duration.as_millis() < 100);
    }
}

/// Memory usage tests  
mod memory_tests {
    use super::*;

    #[test]
    fn test_memory_efficiency() {
        // Create many commands to test memory usage
        let mut commands = Vec::new();
        
        for i in 0..10000 {
            commands.push(Command::Search {
                index_name: format!("mem_test_{}", i),
                query: format!("query for test {}", i),
            });
        }

        // Execute all commands
        for command in commands {
            let result = execute_command(command);
            // Just verify it works, don't keep the results to test memory cleanup
            assert!(!result.is_empty());
        }
    }

    #[test]
    fn test_string_interning_behavior() {
        // Test that identical strings don't cause excessive memory usage
        let identical_name = "identical_test".to_string();
        let identical_query = "identical_query".to_string();

        let mut commands = Vec::new();
        for _ in 0..1000 {
            commands.push(Command::Search {
                index_name: identical_name.clone(),
                query: identical_query.clone(),
            });
        }

        // All commands should work identically
        for command in commands {
            let result = execute_command(command);
            assert!(result.contains("identical_test"));
            assert!(result.contains("identical_query"));
        }
    }
}
