use super::*;

pub struct PlainTextFormatter;

impl ResultFormatter for PlainTextFormatter {
    fn format(&self, output: CommandOutput) -> String {
        match output {
            CommandOutput::Success(sucess_message) => sucess_message,
            CommandOutput::Error(error_message) => error_message,
            CommandOutput::List(indexes) => indexes
                .iter()
                .map(|index| {
                    format!(
                        "{} {} {}",
                        index.index_name, index.index_path, index.target_path
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
            CommandOutput::Search(results) => results
                .iter()
                .map(|result| format!("{}\n{}\n", result.path, result.snippet,))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}
