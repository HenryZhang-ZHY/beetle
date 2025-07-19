use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc::Crc;
use ignore::WalkBuilder;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIndexMetadata {
    pub path: String,
    pub size: u64,
    pub modified_time: u64,
}

const MAGIC: &[u8; 4] = b"BTLX";
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20; // 12 bytes for header + 8 bytes for checksum
const CRC64_ECMA: Crc<u64> = Crc::<u64>::new(&crc::CRC_64_ECMA_182);

pub fn encode(records: &[FileIndexMetadata]) -> Result<Vec<u8>> {
    let estimated_capacity = records.iter().fold(
        HEADER_SIZE,
        |acc, record| acc + 18 + record.path.len(), // Fixed fields (16) + path length (2) + path bytes
    );
    let mut writer = Vec::with_capacity(estimated_capacity);

    let mut digest = CRC64_ECMA.digest();

    // Write header
    writer.write_all(MAGIC)?;
    writer.write_u32::<BigEndian>(VERSION)?;
    writer.write_u32::<BigEndian>(records.len() as u32)?;

    // Write entries
    for record in records {
        writer.write_u64::<BigEndian>(record.size)?;
        writer.write_u64::<BigEndian>(record.modified_time)?;

        let path_bytes = record.path.as_bytes();
        if path_bytes.len() > u16::MAX as usize {
            return Err(anyhow!("Path too long: {} bytes", path_bytes.len()));
        }

        writer.write_u16::<BigEndian>(path_bytes.len() as u16)?;
        writer.write_all(path_bytes)?;
    }

    // Calculate incremental checksum
    digest.update(&writer);
    let checksum = digest.finalize();
    writer.write_u64::<BigEndian>(checksum)?;

    Ok(writer)
}

pub fn decode(bytes: &[u8]) -> Result<Vec<FileIndexMetadata>> {
    if bytes.len() < HEADER_SIZE {
        return Err(anyhow!("Invalid file: too short"));
    }

    let mut cursor = Cursor::new(bytes);

    // Read and verify header
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(anyhow!("Invalid magic signature"));
    }

    let version = cursor.read_u32::<BigEndian>()?;
    if version != VERSION {
        return Err(anyhow!("Unsupported version: {}", version));
    }

    let num_entries = cursor.read_u32::<BigEndian>()?;

    // Verify checksum first
    let data_end = bytes.len() - 8;
    let stored_checksum = {
        let mut checksum_cursor = Cursor::new(&bytes[data_end..]);
        checksum_cursor.read_u64::<BigEndian>()?
    };
    let calculated_checksum = CRC64_ECMA.checksum(&bytes[..data_end]);
    if stored_checksum != calculated_checksum {
        return Err(anyhow!("Checksum mismatch"));
    }

    // Read entries with optimized string handling
    let mut records = Vec::with_capacity(num_entries as usize);
    let remaining_bytes = &bytes[cursor.position() as usize..data_end];
    let mut offset = 0;

    for _ in 0..num_entries {
        if offset + 18 > remaining_bytes.len() {
            return Err(anyhow!("Truncated file: insufficient data for entry"));
        }

        // Read fixed-size fields directly from slice
        let size = u64::from_be_bytes([
            remaining_bytes[offset],
            remaining_bytes[offset + 1],
            remaining_bytes[offset + 2],
            remaining_bytes[offset + 3],
            remaining_bytes[offset + 4],
            remaining_bytes[offset + 5],
            remaining_bytes[offset + 6],
            remaining_bytes[offset + 7],
        ]);
        let modified_time = u64::from_be_bytes([
            remaining_bytes[offset + 8],
            remaining_bytes[offset + 9],
            remaining_bytes[offset + 10],
            remaining_bytes[offset + 11],
            remaining_bytes[offset + 12],
            remaining_bytes[offset + 13],
            remaining_bytes[offset + 14],
            remaining_bytes[offset + 15],
        ]);
        let path_len =
            u16::from_be_bytes([remaining_bytes[offset + 16], remaining_bytes[offset + 17]])
                as usize;

        offset += 18;

        if offset + path_len > remaining_bytes.len() {
            return Err(anyhow!("Truncated file: insufficient data for path"));
        }

        // Validate UTF-8 and create string from slice
        let path = std::str::from_utf8(&remaining_bytes[offset..offset + path_len])
            .map_err(|e| anyhow!("Invalid UTF-8 in path: {}", e))?
            .to_string();

        offset += path_len;

        records.push(FileIndexMetadata {
            path,
            size,
            modified_time,
        });
    }

    Ok(records)
}

pub struct Delta {
    pub added: Vec<FileIndexMetadata>,
    pub modified: Vec<FileIndexMetadata>,
    pub removed: Vec<FileIndexMetadata>,
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

pub fn scan(root_path: &str) -> Vec<FileIndexMetadata> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let walker = WalkBuilder::new(root_path).build_parallel();

    walker.run(|| {
        let results = Arc::clone(&results);
        Box::new(move |entry| {
            process_entry(entry, &results);
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
        modified_time: get_modified_time(&metadata),
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

#[cfg(test)]
mod tests {
    use super::*;

    mod binary_codec {
        use super::*;

        #[test]
        fn test_encode_decode_empty() {
            let records = vec![];

            let encoded = encode(&records).unwrap();
            let decoded = decode(&encoded).unwrap();

            assert_eq!(decoded, records);
        }

        #[test]
        fn test_encode_decode_single_record() {
            let records = vec![FileIndexMetadata {
                path: "test.txt".to_string(),
                size: 1024,
                modified_time: 1622547800,
            }];

            let encoded = encode(&records).unwrap();
            let decoded = decode(&encoded).unwrap();

            assert_eq!(decoded, records);
        }

        #[test]
        fn test_encode_decode_multiple_records() {
            let records = vec![
                FileIndexMetadata {
                    path: "test1.txt".to_string(),
                    size: 1024,
                    modified_time: 1622547800,
                },
                FileIndexMetadata {
                    path: "src/lib.rs".to_string(),
                    size: 2048,
                    modified_time: 1622547900,
                },
                FileIndexMetadata {
                    path: "docs/README.md".to_string(),
                    size: 512,
                    modified_time: 1622548000,
                },
            ];

            let encoded = encode(&records).unwrap();
            let decoded = decode(&encoded).unwrap();

            assert_eq!(decoded, records);
        }

        #[test]
        fn test_unicode_paths() {
            let records = vec![
                FileIndexMetadata {
                    path: "测试.txt".to_string(),
                    size: 100,
                    modified_time: 1622547800,
                },
                FileIndexMetadata {
                    path: "файл.rs".to_string(),
                    size: 200,
                    modified_time: 1622547900,
                },
                FileIndexMetadata {
                    path: "文档/自述文件.md".to_string(),
                    size: 300,
                    modified_time: 1622548000,
                },
            ];

            let encoded = encode(&records).unwrap();
            let decoded = decode(&encoded).unwrap();

            assert_eq!(decoded, records);
        }

        #[test]
        fn test_long_path() {
            let long_path = "a".repeat(1000);
            let records = vec![FileIndexMetadata {
                path: long_path.clone(),
                size: 1024,
                modified_time: 1622547800,
            }];

            let encoded = encode(&records).unwrap();
            let decoded = decode(&encoded).unwrap();

            assert_eq!(decoded[0].path, long_path);
        }

        #[test]
        fn test_path_too_long() {
            let too_long_path = "a".repeat(70000); // > u16::MAX
            let records = vec![FileIndexMetadata {
                path: too_long_path,
                size: 1024,
                modified_time: 1622547800,
            }];

            let result = encode(&records);

            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Path too long: 70000 bytes"
            );
        }

        #[test]
        fn test_invalid_magic() {
            let mut bytes = vec![b'X', b'Y', b'Z', b'W']; // Wrong magic
            bytes.extend_from_slice(&1u32.to_be_bytes()); // version
            bytes.extend_from_slice(&0u32.to_be_bytes()); // num entries
            bytes.extend_from_slice(&0u64.to_be_bytes()); // checksum

            let result = decode(&bytes);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Invalid magic signature");
        }

        #[test]
        fn test_invalid_version() {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(MAGIC);
            bytes.extend_from_slice(&999u32.to_be_bytes()); // Wrong version
            bytes.extend_from_slice(&0u32.to_be_bytes()); // num entries
            bytes.extend_from_slice(&0u64.to_be_bytes()); // checksum

            let result = decode(&bytes);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Unsupported version: 999");
        }

        #[test]
        fn test_corrupted_checksum() {
            let records = vec![FileIndexMetadata {
                path: "test.txt".to_string(),
                size: 1024,
                modified_time: 1622547800,
            }];
            let mut encoded = encode(&records).unwrap();

            // Corrupt the last byte (part of checksum)
            let last_idx = encoded.len() - 1;
            encoded[last_idx] = encoded[last_idx].wrapping_add(1);

            let result = decode(&encoded);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Checksum mismatch");
        }
    }

    mod index_differ {
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
