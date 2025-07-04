use tantivy::schema::*;
use tantivy::schema::{Schema, FAST, STORED, STRING};

pub struct IndexSchema;

impl IndexSchema {
    pub fn create() -> Schema {
        let mut schema_builder = Schema::builder();

        let path_options = TextOptions::default()
            .set_indexing_options(TextFieldIndexing::default().set_tokenizer("ngram3"))
            .set_stored();
        schema_builder.add_text_field(Self::PATH_FIELD, path_options);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("ngram3")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field(Self::CONTENT_FIELD, content_options);

        schema_builder.add_text_field(Self::EXTENSION_FIELD, STRING | STORED);
        schema_builder.add_date_field(Self::LAST_MODIFIED_FIELD, FAST | STORED);

        schema_builder.build()
    }

    pub const PATH_FIELD: &'static str = "path";
    pub const CONTENT_FIELD: &'static str = "content";
    pub const EXTENSION_FIELD: &'static str = "extension";
    pub const LAST_MODIFIED_FIELD: &'static str = "last_modified";
}
