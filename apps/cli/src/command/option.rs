use bpaf::*;

use crate::command::OutputFormat;

pub fn index_name() -> impl Parser<String> {
    long("index")
        .short('i')
        .argument::<String>("INDEX_NAME")
        .help("Name of the index to operate on")
}

pub fn format() -> impl Parser<OutputFormat> {
    long("format")
        .argument::<String>("FORMAT")
        .help("Output format: text (default) or json")
        .parse(|s| match s.as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Invalid format. Use 'text' or 'json'"),
        })
        .fallback(OutputFormat::Text)
}
