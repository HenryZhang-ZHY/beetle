use anyhow::Result;
use tantivy::schema::{Field, Schema, Value};
use tantivy::{doc, TantivyDocument};

/// Represents a document in the search index
pub struct Document {
    pub title: String,
    pub body: String,
    pub path: String,
}

impl Document {
    pub fn new(title: String, body: String, path: String) -> Self {
        Self { title, body, path }
    }

    /// Convert to Tantivy document
    pub fn to_tantivy_doc(
        &self,
        title_field: Field,
        body_field: Field,
        path_field: Field,
    ) -> TantivyDocument {
        doc!(
            title_field => self.title.as_str(),
            body_field => self.body.as_str(),
            path_field => self.path.as_str(),
        )
    }

    /// Extract from Tantivy document
    pub fn from_tantivy_doc(doc: &TantivyDocument, schema: &Schema) -> Result<Self> {
        let title_field = schema.get_field("title")?;
        let body_field = schema.get_field("body")?;
        let path_field = schema.get_field("path")?;

        let title = doc
            .get_first(title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let body = doc
            .get_first(body_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let path = doc
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown path")
            .to_string();

        Ok(Self { title, body, path })
    }
}
