use ignore::WalkBuilder;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FileIndexMetadata {
    pub path: String,
    pub size: u64,
    pub modified_time: u64,
}

pub fn diff_file_index_metadata(
    previous: &[FileIndexMetadata],
    current: &[FileIndexMetadata],
) -> Delta {
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    let previous_set: std::collections::HashSet<_> = previous.iter().map(|f| &f.path).collect();

    for file in current {
        if !previous_set.contains(&file.path) {
            added.push(file.clone());
        } else {
            // Check if the file has been modified
            if let Some(prev_file) = previous.iter().find(|f| f.path == file.path) {
                if file.size != prev_file.size || file.modified_time != prev_file.modified_time {
                    modified.push(file.clone());
                }
            }
        }
    }

    for file in previous {
        if !current.iter().any(|f| f.path == file.path) {
            removed.push(file.clone());
        }
    }

    Delta {
        added,
        modified,
        removed,
    }
}

pub struct Delta {
    pub added: Vec<FileIndexMetadata>,
    pub modified: Vec<FileIndexMetadata>,
    pub removed: Vec<FileIndexMetadata>,
}

pub struct FileScanner;
impl FileScanner {
    pub fn scan(&self, root_path: &str) -> Vec<FileIndexMetadata> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let walker = WalkBuilder::new(root_path).build_parallel();

        walker.run(|| {
            let results = Arc::clone(&results);
            Box::new(move |entry| {
                Self::process_entry(entry, &results);
                ignore::WalkState::Continue
            })
        });

        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }

    fn process_entry(
        entry: Result<ignore::DirEntry, ignore::Error>,
        results: &Arc<Mutex<Vec<FileIndexMetadata>>>,
    ) {
        let dir_entry = match entry {
            Ok(entry) => entry,
            Err(_) => return,
        };

        if !dir_entry.file_type().is_some_and(|ft| ft.is_file()) {
            return;
        }

        let metadata = match fs::metadata(dir_entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => return,
        };

        let path_str = match dir_entry.path().to_str() {
            Some(path) => dunce::canonicalize(path)
                .unwrap_or_else(|_| dir_entry.path().to_path_buf())
                .to_string_lossy()
                .to_string(),
            None => return,
        };

        let file_metadata = FileIndexMetadata {
            path: path_str.to_string(),
            size: metadata.len(),
            modified_time: Self::get_modified_time(&metadata),
        };

        if let Ok(mut results) = results.lock() {
            results.push(file_metadata);
        }
    }

    fn get_modified_time(metadata: &fs::Metadata) -> u64 {
        metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    mod index_differ_tests {
        use super::*;

        #[test]
        fn test_file_addition() {
            let snapshot = vec![FileIndexMetadata {
                path: "a.c".to_string(),
                size: 100,
                modified_time: 1622547800,
            }];
            let manifest = vec![
                FileIndexMetadata {
                    path: "a.c".to_string(),
                    size: 100,
                    modified_time: 1622547800,
                },
                FileIndexMetadata {
                    path: "b.c".to_string(),
                    size: 200,
                    modified_time: 1622547800,
                },
            ];

            let delta = diff_file_index_metadata(&snapshot, &manifest);

            assert_eq!(delta.modified.len(), 0);
            assert_eq!(delta.removed.len(), 0);

            assert_eq!(delta.added.len(), 1);
            assert_eq!(delta.added[0].path, "b.c");
            assert_eq!(delta.added[0].size, 200);
            assert_eq!(delta.added[0].modified_time, 1622547800);
        }

        #[test]
        fn test_file_modification() {
            let snapshot = vec![FileIndexMetadata {
                path: "a.c".to_string(),
                size: 100,
                modified_time: 1622547800,
            }];
            let manifest = vec![FileIndexMetadata {
                path: "a.c".to_string(),
                size: 150,
                modified_time: 1622547900,
            }];

            let delta = diff_file_index_metadata(&snapshot, &manifest);

            assert_eq!(delta.added.len(), 0);
            assert_eq!(delta.removed.len(), 0);

            assert_eq!(delta.modified.len(), 1);
            assert_eq!(delta.modified[0].path, "a.c");
            assert_eq!(delta.modified[0].size, 150);
            assert_eq!(delta.modified[0].modified_time, 1622547900);
        }

        #[test]
        fn test_file_removal() {
            let snapshot = vec![FileIndexMetadata {
                path: "a.c".to_string(),
                size: 100,
                modified_time: 1622547800,
            }];

            let manifest = vec![];

            let delta = diff_file_index_metadata(&snapshot, &manifest);

            assert_eq!(delta.added.len(), 0);
            assert_eq!(delta.modified.len(), 0);

            assert_eq!(delta.removed.len(), 1);
            assert_eq!(delta.removed[0].path, "a.c");
            assert_eq!(delta.removed[0].size, 100);
            assert_eq!(delta.removed[0].modified_time, 1622547800);
        }
    }
}
