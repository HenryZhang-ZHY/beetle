use super::*;

pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl ResultFormatter for JsonFormatter {
    fn format(&self, output: super::CommandOutput) -> String {
        let json_value = match output {
            CommandOutput::Success(sucess_message) => {
                serde_json::json!({
                    "status": "success",
                    "message": sucess_message
                })
            }
            CommandOutput::Error(error_message) => {
                serde_json::json!({
                    "status": "error",
                    "message": error_message
                })
            }
            CommandOutput::List(indexes) => serde_json::json!({
                "status": "success",
                "payload": indexes
            }),
            CommandOutput::Search(results) => serde_json::json!({
                "status": "success",
                "payload": results
            }),
        };

        if self.pretty {
            serde_json::to_string_pretty(&json_value).unwrap()
        } else {
            serde_json::to_string(&json_value).unwrap()
        }
    }
}
