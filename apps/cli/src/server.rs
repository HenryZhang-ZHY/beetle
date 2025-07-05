// use crate::cli::CliRunResult;
// use axum::{
//     extract::{Path, Query},
//     http::StatusCode,
//     response::Json as ResponseJson,
//     routing::get,
//     Router,
// };
// use engine::IndexManager;
// use serde::{Deserialize, Serialize};
// use std::path::PathBuf;
// use tokio::signal;

// #[derive(Serialize)]
// struct IndexResponse {
//     name: String,
//     path: String,
// }

// #[derive(Serialize)]
// struct IndexDetailResponse {
//     name: String,
//     path: String,
//     metadata: IndexMetadataResponse,
// }

// #[derive(Serialize)]
// struct IndexMetadataResponse {
//     doc_count: u64,
//     size_bytes: u64,
// }

// #[derive(Serialize)]
// struct SearchResponse {
//     query: String,
//     index_name: String,
//     results: Vec<SearchResultItem>,
//     total_results: usize,
//     limit: usize,
// }

// #[derive(Serialize)]
// struct SearchResultItem {
//     path: String,
//     content: String,
//     score: f32,
//     line_number: Option<u32>,
//     snippets: Vec<Snippet>,
// }

// #[derive(Serialize)]
// struct Snippet {
//     start: usize,
//     end: usize,

//     starting_line_number: usize,
//     ending_line_number: usize,

//     jump_to_line_number: usize,

//     lines: Vec<String>,
// }

// #[derive(Serialize)]
// struct ErrorResponse {
//     error: String,
// }

// #[derive(Deserialize)]
// struct SearchQuery {
//     q: String,
//     format: Option<String>,
//     limit: Option<usize>,
// }

// // GET /indexes - List all indexes
// async fn list_indexes() -> ResponseJson<Vec<IndexResponse>> {
//     let beetle_home = get_beetle_home();
//     let index_path = PathBuf::from(beetle_home);
//     let index_manager = IndexManager::new(index_path);

//     match index_manager.list_indexes() {
//         Ok(indexes) => {
//             let response: Vec<IndexResponse> = indexes
//                 .into_iter()
//                 .map(|index_info| IndexResponse {
//                     name: index_info.name,
//                     path: index_info.path.to_string_lossy().to_string(),
//                 })
//                 .collect();
//             ResponseJson(response)
//         }
//         Err(_) => {
//             // Return empty list on error
//             ResponseJson(vec![])
//         }
//     }
// }

// // GET /indexes/{index_name} - Get specific index details
// async fn get_index_details(
//     Path(index_name): Path<String>,
// ) -> Result<ResponseJson<IndexDetailResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
//     let beetle_home = get_beetle_home();
//     let index_path = PathBuf::from(beetle_home);
//     let index_manager = IndexManager::new(index_path);

//     match index_manager.list_indexes() {
//         Ok(indexes) => {
//             if let Some(index_info) = indexes.into_iter().find(|idx| idx.name == index_name) {
//                 let response = IndexDetailResponse {
//                     name: index_info.name,
//                     path: index_info.path.to_string_lossy().to_string(),
//                     metadata: IndexMetadataResponse {
//                         doc_count: index_info.metadata.doc_count,
//                         size_bytes: index_info.metadata.size_bytes,
//                     },
//                 };
//                 Ok(ResponseJson(response))
//             } else {
//                 Err((
//                     StatusCode::NOT_FOUND,
//                     ResponseJson(ErrorResponse {
//                         error: format!("Index '{}' not found", index_name),
//                     }),
//                 ))
//             }
//         }
//         Err(_) => Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             ResponseJson(ErrorResponse {
//                 error: "Failed to list indexes".to_string(),
//             }),
//         )),
//     }
// }

// // GET /indexes/{index_name}/search - Search within a specific index
// async fn search_index(
//     Path(index_name): Path<String>,
//     Query(params): Query<SearchQuery>,
// ) -> Result<ResponseJson<SearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
//     let beetle_home = get_beetle_home();
//     let index_path = PathBuf::from(&beetle_home)
//         .join("indexes")
//         .join(&index_name);
//     let index_manager = IndexManager::new(index_path);

//     // Check if index exists first
//     let beetle_index_path = PathBuf::from(beetle_home);
//     let beetle_index_manager = IndexManager::new(beetle_index_path);

//     match beetle_index_manager.list_indexes() {
//         Ok(indexes) => {
//             if !indexes.iter().any(|idx| idx.name == index_name) {
//                 return Err((
//                     StatusCode::NOT_FOUND,
//                     ResponseJson(ErrorResponse {
//                         error: format!("Index '{}' not found", index_name),
//                     }),
//                 ));
//             }
//         }
//         Err(_) => {
//             return Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 ResponseJson(ErrorResponse {
//                     error: "Failed to check index existence".to_string(),
//                 }),
//             ));
//         }
//     }

//     // Perform search
//     match index_manager.search(&params.q) {
//         Ok(search_results) => {
//             let limit = params.limit.unwrap_or(10);
//             let results: Vec<SearchResultItem> = search_results
//                 .into_iter()
//                 .take(limit)
//                 .map(|result| SearchResultItem {
//                     path: result.path,
//                     content: result.snippet,
//                     score: result.score,
//                     line_number: None, // Not available in current SearchResult
//                     snippets: vec![],
//                 })
//                 .collect();

//             let total_results = results.len();

//             let response = SearchResponse {
//                 query: params.q,
//                 index_name,
//                 results,
//                 total_results,
//                 limit,
//             };

//             Ok(ResponseJson(response))
//         }
//         Err(_) => Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             ResponseJson(ErrorResponse {
//                 error: "Search failed: internal error".to_string(),
//             }),
//         )),
//     }
// }

// async fn shutdown_signal() {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("failed to install Ctrl+C handler");
//     };

//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }

//     println!("Received shutdown signal, stopping server gracefully...");
// }

// fn get_beetle_home() -> String {
//     std::env::var("BEETLE_HOME").unwrap_or_else(|_| {
//         let home_dir = std::env::var("HOME")
//             .or_else(|_| std::env::var("USERPROFILE"))
//             .unwrap_or_else(|_| ".".to_string());
//         format!("{}/.beetle", home_dir)
//     })
// }

// pub struct HttpServer;

// impl HttpServer {
//     /// Start the HTTP server on the specified port
//     pub fn start(port: u16) -> CliRunResult {
//         // Since we can't make this method async, we'll use a runtime
//         let rt = tokio::runtime::Runtime::new().unwrap();

//         rt.block_on(async move {
//             // Build our application with RESTful routes
//             let app = Router::new()
//                 .route("/", get(|| async { "hello world" }))
//                 .route("/indexes", get(list_indexes))
//                 .route("/indexes/{index_name}", get(get_index_details))
//                 .route("/indexes/{index_name}/search", get(search_index));

//             // Create the address
//             let addr = format!("{}:{}", "localhost", port);

//             // Create a TCP listener
//             let listener = match tokio::net::TcpListener::bind(&addr).await {
//                 Ok(listener) => listener,
//                 Err(e) => {
//                     return CliRunResult::PlainTextResult(format!(
//                         "Failed to bind to {}: {}",
//                         addr, e
//                     ));
//                 }
//             };

//             println!("Server running on http://{}", addr);

//             // Start the server with graceful shutdown
//             let result = axum::serve(listener, app)
//                 .with_graceful_shutdown(shutdown_signal())
//                 .await;

//             match result {
//                 Ok(_) => CliRunResult::PlainTextResult("Server stopped gracefully".to_string()),
//                 Err(e) => CliRunResult::PlainTextResult(format!("Server error: {}", e)),
//             }
//         })
//     }
// }
