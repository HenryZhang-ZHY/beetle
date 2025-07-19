use crate::cli::get_beetle_home;
use crate::cli::CommandOutput;
use crate::static_files::serve_static_file;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use engine::search::SearchResultItem;
use engine::storage::FsStorage;
use engine::IndexCatalog;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
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
struct SearchResponse {
    query: String,
    index_name: String,
    results: Vec<SearchResultItem>,
    total_results: usize,
    duration_ms: f64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Deserialize)]
struct CreateIndexRequest {
    name: String,
    path: String,
}

#[derive(Clone)]
struct AppState {
    catalog: Arc<IndexCatalog>,
}

async fn list_indexes(
    State(state): State<AppState>,
) -> ResponseJson<Vec<IndexResponse>> {
    match state.catalog.list() {
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
    State(state): State<AppState>,
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexDetailResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    match state.catalog.get_matadata(&index_name) {
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
                error: format!("Index '{index_name}' not found"),
            }),
        )),
    }
}

async fn search_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<ResponseJson<SearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let query = params.q;

    match state.catalog.get_searcher(&index_name) {
        Ok(searcher) => {
            let start_time = std::time::Instant::now();
            let results = searcher.search(&query).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ErrorResponse {
                        error: format!("Search failed: {e}"),
                    }),
                )
            })?;
            let duration = start_time.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;
            
            let total_results = results.len();
            let response = SearchResponse {
                query: query.clone(),
                index_name: index_name.clone(),
                results,
                total_results,
                duration_ms,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Search failed: {e}"),
            }),
        )),
    }
}

async fn create_index(
    State(state): State<AppState>,
    ResponseJson(payload): ResponseJson<CreateIndexRequest>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Validate path exists
    let target_path = std::path::Path::new(&payload.path);
    if !target_path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: format!("Path does not exist: {}", payload.path),
            }),
        ));
    }

    if !target_path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: format!("Path is not a directory: {}", payload.path),
            }),
        ));
    }

    // Check if index already exists
    match state.catalog.list() {
        Ok(existing_indexes) => {
            if existing_indexes
                .iter()
                .any(|idx| idx.index_name == payload.name)
            {
                return Err((
                    StatusCode::CONFLICT,
                    ResponseJson(ErrorResponse {
                        error: format!("Index '{}' already exists", payload.name),
                    }),
                ));
            }
        }
        Err(_) => {
            // Continue with creation if we can't list existing indexes
        }
    }

    match state.catalog.create(&payload.name, &payload.path) {
        Ok(_) => {
            let response = IndexResponse {
                name: payload.name,
                path: payload.path,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Failed to create index: {e}"),
            }),
        )),
    }
}

async fn reindex_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Get existing index metadata to retrieve the target path
    let metadata = match state.catalog.get_matadata(&index_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse {
                    error: format!("Index '{index_name}' not found"),
                }),
            ));
        }
    };

    // Reset the index (clear existing data)
    match state.catalog.reset(&index_name) {
        Ok(_) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to reset index: {e}"),
                }),
            ));
        }
    }

    // Create a new writer to rebuild the index
    let mut writer = match state.catalog.get_writer(&index_name) {
        Ok(writer) => writer,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to create index writer: {e}"),
                }),
            ));
        }
    };

    // Build the index from the target path
    match writer.index() {
        Ok(_) => {
            let response = IndexResponse {
                name: index_name.clone(),
                path: metadata.target_path,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Failed to rebuild index: {e}"),
            }),
        )),
    }
}

async fn delete_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let metadata = match state.catalog.get_matadata(&index_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse {
                    error: format!("Index '{index_name}' not found"),
                }),
            ));
        }
    };

    match state.catalog.remove(&index_name) {
        Ok(_) => {
            let response = IndexResponse {
                name: index_name.clone(),
                path: metadata.target_path,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Failed to delete index: {e}"),
            }),
        )),
    }
}

async fn update_index(
    State(state): State<AppState>,
    Path(index_name): Path<String>,
) -> Result<ResponseJson<IndexResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let metadata = match state.catalog.get_matadata(&index_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse {
                    error: format!("Index '{index_name}' not found"),
                }),
            ));
        }
    };

    let mut writer = match state.catalog.get_writer(&index_name) {
        Ok(writer) => writer,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: format!("Failed to create index writer: {e}"),
                }),
            ));
        }
    };

    match writer.index() {
        Ok(_) => {
            let response = IndexResponse {
                name: index_name.clone(),
                path: metadata.target_path,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: format!("Failed to update index: {e}"),
            }),
        )),
    }
}

pub struct HttpServer;

impl HttpServer {
    pub fn start(port: u16) -> CommandOutput {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async move {
            // Create shared catalog once
            let beetle_home_path = PathBuf::from(get_beetle_home());
            let storage = FsStorage::new(beetle_home_path);
            let catalog = IndexCatalog::new(storage);
            let app_state = AppState { catalog: Arc::new(catalog) };

            let app = Router::new()
                .route("/api/indexes", get(list_indexes).post(create_index))
                .route(
                    "/api/indexes/{index_name}",
                    get(get_index_details).delete(delete_index),
                )
                .route("/api/indexes/{index_name}/search", get(search_index))
                .route("/api/indexes/{index_name}/reindex", post(reindex_index))
                .route("/api/indexes/{index_name}/update", post(update_index))
                .fallback(serve_static_file)
                .with_state(app_state);

            let address = format!("{}:{}", "localhost", port);
            let listener = match tokio::net::TcpListener::bind(&address).await {
                Ok(listener) => listener,
                Err(e) => {
                    return CommandOutput::Error(format!("Failed to bind to {address}: {e}"));
                }
            };
            println!("Server running on http://{address}");

            let result = axum::serve(listener, app)
                .with_graceful_shutdown(Self::shutdown_signal())
                .await;
            match result {
                Ok(_) => CommandOutput::Success("Server stopped gracefully".to_string()),
                Err(e) => CommandOutput::Error(format!("Server error: {e}")),
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
