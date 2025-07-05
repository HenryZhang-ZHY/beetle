use crate::index_storage::{IndexStorage, IndexStorageMetadata};
use crate::index_writter::IndexWriter;

pub struct IndexCatalog {
    storage: Box<dyn IndexStorage>,
}

impl IndexCatalog {
    pub fn new<T: IndexStorage + 'static>(storage: T) -> Self {
        IndexCatalog {
            storage: Box::new(storage),
        }
    }

    pub fn create(&self, index_name: &str, target_path: &str) -> Result<(), String> {
        self.storage
            .create(index_name, target_path)
            .map_err(|e| format!("Failed to create index {}: {}", index_name, e))?;

        Ok(())
    }

    pub fn get_writer(&self, index_name: &str) -> Result<IndexWriter, String> {
        let metadata = self
            .storage
            .get_metadata(index_name)
            .map_err(|e| format!("Failed to get metadata for index {}: {}", index_name, e))?;

        let index = self
            .storage
            .open(index_name)
            .map_err(|e| format!("Failed to open index {}: {}", index_name, e))?;

        let writer = IndexWriter::new(self.storage.as_ref(), metadata, index).map_err(|e| {
            format!(
                "Failed to create index writer for index {}: {}",
                index_name, e
            )
        })?;

        Ok(writer)
    }

    pub fn remove(&self, index_name: &str) -> Result<(), String> {
        self.storage
            .remove(index_name)
            .map_err(|e| format!("Failed to remove index {}: {}", index_name, e))?;

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<IndexStorageMetadata>, String> {
        self.storage.list()
    }
}
