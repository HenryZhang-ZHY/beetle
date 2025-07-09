use crate::cli::get_beetle_home;
use crate::cli::CommandOutput;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::get,
    Router,
};
use engine::search::SearchResultItem;
use engine::storage::FsStorage;
use engine::IndexCatalog;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::signal;

#[derive(Serialize)]
struct IndexResponse {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct IndexDetailResponse {
    index_name: String,
    index_path: String,
    target_path: String,
}

#[derive(Serialize)]
struct IndexMetadataResponse {
    doc_count: u64,
    size_bytes: u64,
}

#[derive(Serialize)]
struct SearchResponse {
    query: String,
    index_name: String,
    results: Vec<SearchResultItem>,
    total_results: usize,
}

#[derive(Serialize)]
struct Snippet {
    start: usize,
    end: usize,

    starting_line_number: usize,
    ending_line_number: usize,

    jump_to_line_number: usize,

    lines: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

fn create_index_catalog() -> IndexCatalog {
    let beetl_home_path = PathBuf::from(get_beetle_home());
    let storage = FsStorage::new(beetl_home_path);

    IndexCatalog::new(storage)
}

async fn list_indexes() -> ResponseJson<Vec<IndexResponse>> {
    let catalog = create_index_catalog();

    match catalog.list() {
        Ok(indexes) => {
            let response: Vec<IndexResponse> = indexes
                .into_iter()
                .map(|index| IndexResponse {
                    name: index.index_name,
                    path: index.index_path,
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

async fn get_index_details(
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexDetailResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let catalog = create_index_catalog();

    match catalog.get_matadata(&index_name) {
        Ok(metadata) => {
            let response = IndexDetailResponse {
                index_name: metadata.index_name.clone(),
                index_path: metadata.index_path.clone(),
                target_path: metadata.target_path.clone(),
            };
            Ok(ResponseJson(response))
        }
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            ResponseJson(ErrorResponse {
                error: format!("Index '{}' not found", index_name),
            }),
        )),
    }
}

async fn search_index(
    Path(index_name): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<ResponseJson<SearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let catalog = create_index_catalog();

    let query = params.q;

    match catalog.get_searcher(&index_name) {
        Ok(searcher) => {
            let results = searcher.search(&query).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ErrorResponse {
                        error: format!("Search failed: {}", e),
                    }),
                )
            })?;
            let total_results = results.len();
            let response = SearchResponse {
                query: query.clone(),
                index_name: index_name.clone(),
                results,
                total_results,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Search failed: {}", e),
            }),
        )),
    }
}

pub struct HttpServer;

impl HttpServer {
    pub fn start(port: u16) -> CommandOutput {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async move {
            let app = Router::new()
                .route("/api/indexes", get(list_indexes))
                .route("/api/indexes/{index_name}", get(get_index_details))
                .route("/api/indexes/{index_name}/search", get(search_index));

            let address = format!("{}:{}", "localhost", port);
            let listener = match tokio::net::TcpListener::bind(&address).await {
                Ok(listener) => listener,
                Err(e) => {
                    return CommandOutput::Error(format!("Failed to bind to {}: {}", address, e));
                }
            };
            println!("Server running on http://{}", address);

            let result = axum::serve(listener, app)
                .with_graceful_shutdown(Self::shutdown_signal())
                .await;
            match result {
                Ok(_) => CommandOutput::Success("Server stopped gracefully".to_string()),
                Err(e) => CommandOutput::Error(format!("Server error: {}", e)),
            }
        })
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("Received shutdown signal, stopping server gracefully...");
    }
}
