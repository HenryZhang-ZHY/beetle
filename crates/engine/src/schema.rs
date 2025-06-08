use std::time::SystemTime;

use tantivy::schema::*;
use tantivy::TantivyDocument;

#[allow(dead_code)]
pub struct CodeIndexSchema {
    pub schema: Schema,
    pub path: Field,
    pub content: Field,
    pub extension: Field,
    pub last_modified: Field,
}

impl CodeIndexSchema {
    pub fn new() -> CodeIndexSchema {
        let mut schema_builder = Schema::builder();

        let path = schema_builder.add_text_field(Self::PATH_FIELD, STRING | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("ngram3")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let content = schema_builder.add_text_field(Self::CONTENT_FIELD, content_options);

        let extension = schema_builder.add_text_field(Self::EXTENSION_FIELD, STRING | STORED);
        let last_modified = schema_builder.add_date_field(Self::LAST_MODIFIED_FIELD, FAST | STORED);

        Self {
            schema: schema_builder.build(),
            path,
            content,
            extension,
            last_modified,
        }
    }

    pub const PATH_FIELD: &'static str = "path";
    pub const CONTENT_FIELD: &'static str = "content";
    pub const EXTENSION_FIELD: &'static str = "extension";
    pub const LAST_MODIFIED_FIELD: &'static str = "last_modified";
}

pub struct CodeIndexDocument {
    pub path: String,
    pub content: String,
    pub extension: String,
    pub last_modified: SystemTime,
}

impl CodeIndexDocument {
    pub fn from_path(path: &String) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let extension = std::path::PathBuf::from(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_string();
        let last_modified = std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::now());

        CodeIndexDocument {
            path: path.clone(),
            content,
            extension,
            last_modified,
        }
    }

    pub fn to_tantivy_document(&self, schema: &Schema) -> TantivyDocument {
        let mut doc = TantivyDocument::new();
        doc.add_text(
            schema.get_field(CodeIndexSchema::PATH_FIELD).unwrap(),
            &self.path,
        );
        doc.add_text(
            schema.get_field(CodeIndexSchema::CONTENT_FIELD).unwrap(),
            &self.content,
        );
        doc.add_text(
            schema.get_field(CodeIndexSchema::EXTENSION_FIELD).unwrap(),
            &self.extension,
        );

        let last_modified = self
            .last_modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        doc.add_date(
            schema
                .get_field(CodeIndexSchema::LAST_MODIFIED_FIELD)
                .unwrap(),
            tantivy::DateTime::from_timestamp_secs(last_modified),
        );
        doc
    }
}
