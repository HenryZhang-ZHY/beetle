use tantivy::schema::{Schema, STORED, TEXT};

pub struct IndexSchema;

impl IndexSchema {
    pub fn create() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field(Self::CONTENT_FIELD, TEXT | STORED);
        schema_builder.add_text_field(Self::PATH_FIELD, TEXT | STORED);
        schema_builder.build()
    }

    pub const CONTENT_FIELD: &'static str = "content";
    pub const PATH_FIELD: &'static str = "path";
}
