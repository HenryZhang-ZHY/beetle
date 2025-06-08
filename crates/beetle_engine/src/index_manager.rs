use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::{Index, IndexWriter, ReloadPolicy};
use walkdir::WalkDir;

use crate::document::Document;
use crate::query::{SearchOptions, SearchResult};
use crate::utils::is_text_file;

/// Manages search indexes
pub struct IndexManager {
    base_path: PathBuf,
}

impl Default for IndexManager {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }
}

impl IndexManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Create a new index from a repository
    pub fn create_index(&self, index_name: &str, repo_path: &PathBuf) -> Result<IndexingStats> {
        let index_path = self.base_path.join(index_name);
        fs::create_dir_all(&index_path).with_context(|| {
            format!("Failed to create index directory: {}", index_path.display())
        })?;

        let schema = Self::create_schema();
        let index = Index::create_in_dir(&index_path, schema.clone())
            .with_context(|| "Failed to create tantivy index")?;

        let mut writer = index
            .writer(50_000_000)
            .with_context(|| "Failed to create index writer")?;

        let stats = self.index_repository(&mut writer, &schema, repo_path)?;

        writer.commit().with_context(|| "Failed to commit index")?;

        Ok(stats)
    }

    /// Index all files in a repository
    fn index_repository(
        &self,
        writer: &mut IndexWriter,
        schema: &Schema,
        repo_path: &PathBuf,
    ) -> Result<IndexingStats> {
        let title_field = schema.get_field("title")?;
        let body_field = schema.get_field("body")?;
        let path_field = schema.get_field("path")?;

        let mut stats = IndexingStats::default();

        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            if !is_text_file(file_path) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(file_path) {
                let relative_path = file_path
                    .strip_prefix(repo_path)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let doc = Document::new(file_name, content.clone(), relative_path);
                writer.add_document(doc.to_tantivy_doc(title_field, body_field, path_field))?;

                stats.file_count += 1;
                stats.total_size += content.len() as u64;

                if stats.file_count % 100 == 0 {
                    println!("Indexed {} files...", stats.file_count);
                }
            }
        }

        Ok(stats)
    }

    /// Search an existing index
    pub fn search(
        &self,
        index_name: &str,
        query_str: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let index_path = self.find_index(index_name)?;
        let index = Index::open_in_dir(&index_path)
            .with_context(|| format!("Failed to open index at: {}", index_path.display()))?;

        let searcher = crate::query::create_searcher(&index)?;
        let results = crate::query::search(&index, &searcher, query_str, options)?;

        Ok(results)
    }

    /// List all available indexes
    pub fn list_indexes(&self) -> Result<Vec<IndexInfo>> {
        let search_paths = vec![
            self.base_path.join("indexes"),
            self.base_path.join("indices"),
            self.base_path.clone(),
        ];

        let mut found_indexes = Vec::new();

        for search_path in search_paths {
            if search_path.exists() && search_path.is_dir() {
                self.scan_directory_for_indexes(&search_path, &mut found_indexes)?;
            }
        }

        Ok(found_indexes)
    }

    /// Find an index by name
    fn find_index(&self, index_name: &str) -> Result<PathBuf> {
        let possible_paths = vec![
            PathBuf::from(index_name),
            self.base_path.join("indexes").join(index_name),
            self.base_path.join("indices").join(index_name),
            self.base_path.join(index_name),
        ];

        possible_paths
            .into_iter()
            .find(|p| p.exists() && p.is_dir() && p.join("meta.json").exists())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Index '{}' not found. Tried looking in current directory and common index locations.",
                    index_name
                )
            })
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

    /// Create the schema for the index
    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);
        schema_builder.add_text_field("path", STORED);
        schema_builder.build()
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
