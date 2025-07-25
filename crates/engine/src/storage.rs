use crate::change::{self, FileIndexMetadata};
use crate::schema::CodeIndexSchema;
use crate::tokenizers::CodeTokenizer;
use std::fs;
use std::path::PathBuf;
use tantivy::Index;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IndexStorageMetadata {
    pub index_name: String,
    pub index_path: String,
    pub target_path: String,
}

pub trait IndexStorage: Send + Sync {
    fn index_dir(&self) -> String;
    fn create(&self, index_name: &str, target_path: &str) -> Result<Index, String>;
    fn open(&self, index_name: &str) -> Result<Index, String>;
    fn remove(&self, index_name: &str) -> Result<(), String>;
    fn list(&self) -> Result<Vec<IndexStorageMetadata>, String>;
    fn get_metadata(&self, index_name: &str) -> Result<IndexStorageMetadata, String> {
        let list = self.list()?;
        for metadata in list {
            if metadata.index_name == index_name {
                return Ok(metadata);
            }
        }

        Err(format!("Index {index_name} not found"))
    }
    fn reset(&self, index_name: &str) -> Result<(), String> {
        let metadata = self.get_metadata(index_name)?;
        self.remove(index_name)?;
        self.create(&metadata.index_name, &metadata.target_path)?;

        Ok(())
    }
    fn read_file_index_metadata(&self, index_name: &str) -> Result<Vec<FileIndexMetadata>, String>;
    fn save_file_index_metadata(
        &self,
        index_name: &str,
        metadata: Vec<FileIndexMetadata>,
    ) -> Result<(), String>;
}

pub struct FsStorage {
    pub root: PathBuf,
}

impl FsStorage {
    pub fn new(root: PathBuf) -> Self {
        FsStorage { root }
    }

    fn get_file_index_path(&self, index_name: &str) -> Result<PathBuf, String> {
        let index_metadata = self.get_metadata(index_name)?;
        let file_index_path =
            PathBuf::from(&index_metadata.index_path).join(Self::FILE_INDEX_SNAPSHOT_FILE_NAME);

        Ok(file_index_path)
    }

    pub const META_JSON_FILE_NAME: &'static str = "meta.json";
    pub const FILE_INDEX_SNAPSHOT_FILE_NAME: &'static str = "file_index_snapshot.bin";
}

impl IndexStorage for FsStorage {
    fn index_dir(&self) -> String {
        self.root.to_string_lossy().to_string()
    }

    fn create(&self, index_name: &str, target_path: &str) -> Result<Index, String> {
        let index_root_path = self.root.join(index_name);
        let absolute_index_root_path = dunce::canonicalize(self.root.join(index_name))
            .unwrap_or_else(|_| PathBuf::from(&index_root_path));
        if absolute_index_root_path.exists() {
            return Err(format!("Index {index_name} already exists"));
        }
        fs::create_dir_all(&absolute_index_root_path)
            .map_err(|e| format!("Failed to create index directory {index_name}: {e}"))?;

        let absolute_target_path = dunce::canonicalize(PathBuf::from(target_path))
            .unwrap_or_else(|_| PathBuf::from(target_path));
        if !absolute_target_path.exists() {
            return Err(format!(
                "Target path '{}' does not exist",
                absolute_target_path.to_string_lossy()
            ));
        }
        let metadata = IndexStorageMetadata {
            index_name: index_name.to_string(),
            index_path: absolute_index_root_path.to_string_lossy().to_string(),
            target_path: absolute_target_path.to_string_lossy().to_string(),
        };
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| format!("Failed to serialize metadata for index {index_name}: {e}"))?;
        let metadata_path = absolute_index_root_path.join(Self::META_JSON_FILE_NAME);
        fs::write(&metadata_path, metadata_json)
            .map_err(|e| format!("Failed to write metadata file for index {index_name}: {e}"))?;

        let index_path = absolute_index_root_path.join("index");
        fs::create_dir_all(&index_path)
            .map_err(|e| format!("Failed to create index directory {index_name}: {e}"))?;
        let index = Index::create_in_dir(&index_path, CodeIndexSchema::new().schema)
            .map_err(|e| format!("Failed to create index {index_name}: {e}"))?;
        index
            .tokenizers()
            .register("code", CodeTokenizer::default());

        Ok(index)
    }

    fn open(&self, index_name: &str) -> Result<Index, String> {
        let index_path = self.root.join(index_name).join("index");
        if !index_path.exists() {
            return Err(format!("Index {index_name} does not exist"));
        }

        let index = Index::open_in_dir(&index_path)
            .map_err(|e| format!("Failed to open index {index_name}: {e}"))?;
        index
            .tokenizers()
            .register("code", CodeTokenizer::default());

        Ok(index)
    }

    fn remove(&self, index_name: &str) -> Result<(), String> {
        let index_path = self.root.join(index_name);
        if index_path.exists() {
            fs::remove_dir_all(&index_path)
                .map_err(|e| format!("Failed to remove index {index_name}: {e}"))?;
            Ok(())
        } else {
            Err(format!("Index {index_name} does not exist"))
        }
    }

    fn list(&self) -> Result<Vec<IndexStorageMetadata>, String> {
        let mut indices = Vec::new();

        let entries =
            fs::read_dir(&self.root).map_err(|e| format!("Failed to read index directory: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to get file type: {e}"))?;
            if !file_type.is_dir() {
                continue;
            }

            let index_metadata_path = entry.path().join(Self::META_JSON_FILE_NAME);
            if !index_metadata_path.exists() {
                return Err(format!(
                    "Metadata file does not exist for index {}",
                    entry.file_name().to_string_lossy()
                ));
            }

            let metadata_json = fs::read_to_string(&index_metadata_path)
                .map_err(|e| format!("Failed to read metadata file: {e}"))?;
            let metadata: IndexStorageMetadata = serde_json::from_str(&metadata_json)
                .map_err(|e| format!("Failed to parse metadata JSON: {e}"))?;

            indices.push(metadata);
        }

        indices.sort_by(|a, b| a.index_name.cmp(&b.index_name));

        Ok(indices)
    }

    fn save_file_index_metadata(
        &self,
        index_name: &str,
        metadata: Vec<FileIndexMetadata>,
    ) -> Result<(), String> {
        let file_index_path = self.get_file_index_path(index_name)?;
        let bytes = change::encode(&metadata)
            .map_err(|e| format!("Failed to encode file index metadata: {e}"))?;
        fs::write(&file_index_path, bytes)
            .map_err(|e| format!("Failed to write file index metadata to {file_index_path:?}: {e}"))
    }

    fn read_file_index_metadata(&self, index_name: &str) -> Result<Vec<FileIndexMetadata>, String> {
        let file_index_path = self.get_file_index_path(index_name)?;
        if !file_index_path.exists() {
            return Ok(Vec::new());
        }

        let bytes = fs::read(&file_index_path).map_err(|e| {
            format!("Failed to read file index metadata from {file_index_path:?}: {e}")
        })?;
        change::decode(&bytes).map_err(|e| {
            format!("Failed to decode file index metadata from {file_index_path:?}: {e}")
        })
    }
}
