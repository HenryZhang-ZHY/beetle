use anyhow::Result;
use tantivy::schema::{Field, Schema, Value};
use tantivy::{doc, TantivyDocument};

/// Represents a document in the search index
pub struct Document {
    pub content: String,
    pub path: String,
}

use crate::schema::IndexSchema;

impl Document {
    pub fn new(content: String, path: String) -> Self {
        Self { content, path }
    }

    /// Convert to Tantivy document
    pub fn to_tantivy_doc(&self, content_field: Field, path_field: Field) -> TantivyDocument {
        doc!(
            content_field => self.content.as_str(),
            path_field => self.path.as_str(),
        )
    }

    /// Extract from Tantivy document
    pub fn from_tantivy_doc(doc: &TantivyDocument, schema: &Schema) -> Result<Self> {
        let content_field = schema.get_field(IndexSchema::CONTENT_FIELD)?;
        let path_field = schema.get_field(IndexSchema::PATH_FIELD)?;

        let content = doc
            .get_first(content_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let path = doc
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown path")
            .to_string();

        Ok(Self { content, path })
    }
}
