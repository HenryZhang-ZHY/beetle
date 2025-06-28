use crate::cli::CliRunResult;
use axum::{routing::get, Router, response::Json as ResponseJson};
use beetle_engine::IndexManager;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct IndexResponse {
    name: String,
    path: String,
}

async fn list_indexes() -> ResponseJson<Vec<IndexResponse>> {
    let beetle_home = get_beetle_home();
    let index_path = PathBuf::from(beetle_home);
    let index_manager = IndexManager::new(index_path);
    
    match index_manager.list_indexes() {
        Ok(indexes) => {
            let response: Vec<IndexResponse> = indexes
                .into_iter()
                .map(|index_info| IndexResponse {
                    name: index_info.name,
                    path: index_info.path.to_string_lossy().to_string(),
                })
                .collect();
            ResponseJson(response)
        }
        Err(_) => {
            // Return empty list on error
            ResponseJson(vec![])
        }
    }
}

fn get_beetle_home() -> String {
    std::env::var("BEETLE_HOME").unwrap_or_else(|_| {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{}/.beetle", home_dir)
    })
}

pub struct HttpServer;

impl HttpServer {
    /// Start the HTTP server on the specified port
    pub fn start(port: u16) -> CliRunResult {
        // Since we can't make this method async, we'll use a runtime
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            // Build our application with routes
            let app = Router::new()
                .route("/", get(|| async { "hello world" }))
                .route("/index", get(list_indexes));

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
