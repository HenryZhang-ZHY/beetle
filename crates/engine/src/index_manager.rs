use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tantivy::{Index, IndexWriter, ReloadPolicy};

use crate::document::Document;
use crate::schema::CodeIndexSchema;

/// Options for controlling indexing behavior, particularly around git ignore rules
///
/// This struct allows fine-grained control over which files are included when indexing
/// a repository. By default, it respects standard git ignore patterns to avoid indexing
/// files that are typically not meant to be searched (like build artifacts, dependencies, etc.).
///
/// # Examples
///
/// ```rust
/// use engine::IndexingOptions;
///
/// // Default behavior - respects .gitignore files
/// let options = IndexingOptions::new();
///
/// // Ignore all git rules and index everything
/// let options = IndexingOptions::ignore_git_rules();
///
/// // Custom configuration
/// let options = IndexingOptions {
///     respect_gitignore: true,
///     include_hidden: true,    // Include hidden files even if not in .gitignore
///     respect_git_global: false,  // Ignore global git config
///     respect_git_exclude: true,
/// };
/// ```
#[derive(Default, Debug, Clone)]
pub struct IndexingOptions {
    /// Whether to respect .gitignore files (default: true)
    ///
    /// When true, files and directories listed in .gitignore files will be skipped
    /// during indexing. This is usually desired to avoid indexing build artifacts,
    /// node_modules, .git directories, etc.
    pub respect_gitignore: bool,

    /// Whether to include hidden files (default: false)
    ///
    /// When false, files and directories starting with '.' are skipped unless
    /// explicitly allowed by git ignore rules. When true, hidden files are included.
    pub include_hidden: bool,

    /// Whether to respect global git ignore (default: true)
    ///
    /// When true, patterns from the global git ignore file (usually ~/.gitignore_global)
    /// are respected during indexing.
    pub respect_git_global: bool,

    /// Whether to respect .git/info/exclude (default: true)
    ///
    /// When true, patterns from .git/info/exclude are respected. This file contains
    /// repository-specific ignore patterns that are not shared with other developers.
    pub respect_git_exclude: bool,
}

impl IndexingOptions {
    /// Create default indexing options
    ///
    /// Default behavior:
    /// - Respects .gitignore files
    /// - Excludes hidden files
    /// - Respects global git ignore
    /// - Respects .git/info/exclude
    pub fn new() -> Self {
        Self {
            respect_gitignore: true,
            include_hidden: false,
            respect_git_global: true,
            respect_git_exclude: true,
        }
    }

    /// Create options that ignore all git ignore rules
    ///
    /// This will index all files in the repository, including those typically
    /// ignored by git (build artifacts, dependencies, etc.). Use with caution
    /// as this may result in very large indexes with many irrelevant files.
    pub fn ignore_git_rules() -> Self {
        Self {
            respect_gitignore: false,
            include_hidden: true,
            respect_git_global: false,
            respect_git_exclude: false,
        }
    }
}

/// Manages search indexes
pub struct IndexManager {
    pub index_path: PathBuf,
}

impl Default for IndexManager {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("."),
        }
    }
}

impl IndexManager {
    pub fn new(index_path: PathBuf) -> Self {
        Self { index_path }
    }

    pub fn new_index(
        &self,
        _index_name: &str,
        path_to_be_indexed: &PathBuf,
        options: Option<IndexingOptions>,
    ) -> Result<IndexingStats> {
        let options = options.unwrap_or_else(IndexingOptions::new);

        fs::create_dir_all(&self.index_path).with_context(|| {
            format!(
                "Failed to create index directory: {}",
                &self.index_path.display()
            )
        })?;

        let schema = CodeIndexSchema::create();
        let index = Index::create_in_dir(&self.index_path, schema.clone())
            .with_context(|| "Failed to create tantivy index")?;

        let mut writer = index
            .writer(1024 * 1024 * 1024 * 2)
            .with_context(|| "Failed to create index writer")?;

        let stats = self.index_repository(&mut writer, &schema, path_to_be_indexed, &options)?;

        writer.commit().with_context(|| "Failed to commit index")?;

        Ok(stats)
    }

    /// Index all files in a repository with git ignore support
    ///
    /// This method walks through the repository directory and indexes all text files
    /// while respecting git ignore patterns based on the provided options.
    ///
    /// The method uses the `ignore` crate which provides robust support for:
    /// - .gitignore files at any level in the directory tree
    /// - Global git ignore files
    /// - .git/info/exclude files
    /// - Hidden file filtering
    ///
    /// # Arguments
    /// * `writer` - Tantivy index writer to add documents to
    /// * `schema` - Index schema defining the document structure
    /// * `repo_path` - Path to the repository root
    /// * `options` - Options controlling which files to include/exclude
    fn index_repository(
        &self,
        writer: &mut IndexWriter,
        schema: &tantivy::schema::Schema,
        path_to_be_indexed: &PathBuf,
        options: &IndexingOptions,
    ) -> Result<IndexingStats> {
        let content_field = schema.get_field(CodeIndexSchema::CONTENT_FIELD)?;
        let path_field = schema.get_field(CodeIndexSchema::PATH_FIELD)?;

        let mut stats = IndexingStats::default();

        // Use ignore crate to respect .gitignore files
        let walker = WalkBuilder::new(path_to_be_indexed)
            .hidden(!options.include_hidden) // Include hidden files based on options
            .git_ignore(options.respect_gitignore) // Respect .gitignore files
            .git_global(options.respect_git_global) // Respect global git ignore
            .git_exclude(options.respect_git_exclude) // Respect .git/info/exclude
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let file_path = entry.path();

            // Skip directories
            if !file_path.is_file() {
                continue;
            }

            if !is_text_file(file_path) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(file_path) {
                let absolute_path = file_path.to_string_lossy().to_string();

                let doc = Document::new(content.clone(), absolute_path);
                writer.add_document(doc.to_tantivy_doc(content_field, path_field))?;

                stats.file_count += 1;
                stats.total_size += content.len() as u64;

                if stats.file_count % 100 == 0 {
                    println!("Indexed {} files...", stats.file_count);
                }
            }
        }

        Ok(stats)
    }
    /// List all available indexes
    pub fn list_indexes(&self) -> Result<Vec<IndexInfo>> {
        let search_paths = vec![
            self.index_path.join("indexes"),
            self.index_path.join("indices"),
            self.index_path.clone(),
        ];

        let mut found_indexes = Vec::new();

        for search_path in search_paths {
            if search_path.exists() && search_path.is_dir() {
                self.scan_directory_for_indexes(&search_path, &mut found_indexes)?;
            }
        }

        Ok(found_indexes)
    }

    /// Scan a directory for indexes
    fn scan_directory_for_indexes(
        &self,
        dir: &PathBuf,
        indexes: &mut Vec<IndexInfo>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.join("meta.json").exists() {
                let index_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let metadata = IndexMetadata::from_path(&path)?;
                indexes.push(IndexInfo {
                    name: index_name,
                    path,
                    metadata,
                });
            }
        }
        Ok(())
    }
}

fn is_text_file(path: &Path) -> bool {
    const BINARY_EXTENSIONS: &[&str] = &[
        "exe", "dll", "so", "dylib", "bin", "obj", "o", "jpg", "jpeg", "png", "gif", "bmp", "ico",
        "mp3", "mp4", "avi", "mov", "wav", "zip", "tar", "gz", "rar", "7z", "pdf", "doc", "docx",
        "xls", "xlsx",
    ];

    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        !BINARY_EXTENSIONS.contains(&ext.as_str())
    } else {
        true // Assume files without extensions might be text
    }
}

#[derive(Default, Debug, Clone)]
pub struct IndexingStats {
    pub file_count: u32,
    pub total_size: u64,
    pub index_name: String,
    pub index_path: PathBuf,
    pub repo_path: PathBuf,
}

/// Information about an index
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub path: PathBuf,
    pub metadata: IndexMetadata,
}

/// Metadata about an index
#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub doc_count: u64,
    pub size_bytes: u64,
}

impl IndexMetadata {
    /// Get metadata from an index path
    pub fn from_path(index_path: &PathBuf) -> Result<Self> {
        let size_bytes = Self::calculate_directory_size(index_path)?;
        let doc_count = Self::get_document_count(index_path).unwrap_or(0);

        Ok(Self {
            doc_count,
            size_bytes,
        })
    }

    fn calculate_directory_size(path: &PathBuf) -> Result<u64> {
        let mut size = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        size += metadata.len();
                    }
                }
            }
        }
        Ok(size)
    }

    fn get_document_count(index_path: &PathBuf) -> Result<u64> {
        let index = Index::open_in_dir(index_path)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        Ok(reader.searcher().num_docs() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(Path::new("test.rs")));
        assert!(is_text_file(Path::new("README.md")));
        assert!(is_text_file(Path::new("file_without_extension")));
        assert!(!is_text_file(Path::new("image.jpg")));
        assert!(!is_text_file(Path::new("binary.exe")));
    }
}
