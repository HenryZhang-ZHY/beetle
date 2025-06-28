use crate::cli::CliRunResult;
use axum::{routing::get, Router};

pub struct HttpServer;

impl HttpServer {
    /// Start the HTTP server on the specified port
    pub fn start(port: u16) -> CliRunResult {
        // Since we can't make this method async, we'll use a runtime
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // Build our application with a simple hello world route
            let app = Router::new().route("/", get(|| async { "hello world" }));

            // Create the address
            let addr = format!("{}:{}", "localhost", port);

            // Create a TCP listener
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    return CliRunResult::PlainTextResult(format!(
                        "Failed to bind to {}: {}",
                        addr, e
                    ));
                }
            };

            println!("Server running on http://{}", addr);

            // Start the server
            if let Err(e) = axum::serve(listener, app).await {
                CliRunResult::PlainTextResult(format!("Server error: {}", e))
            } else {
                CliRunResult::PlainTextResult("Server stopped".to_string())
            }
        })
    }
}
