use tantivy::schema::{Schema, STORED, TEXT};

/// Schema configuration for the search index
pub struct IndexSchema;

impl IndexSchema {
    /// Create the schema for the index
    ///
    /// This schema defines three fields:
    /// - `title`: The filename or document title (searchable and stored)
    /// - `body`: The full content of the file (searchable and stored)
    /// - `path`: The relative file path (stored only, not searchable)
    pub fn create() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.add_text_field("path", STORED);
        schema_builder.build()
    }

    pub const CONTENT_FIELD: &'static str = "content";
    pub const PATH_FIELD: &'static str = "path";
}
