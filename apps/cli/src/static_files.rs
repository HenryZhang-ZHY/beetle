use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::Response;
use include_dir::{include_dir, Dir};
use mime_guess::from_path;

static WEB_UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../apps/webui/dist");

pub async fn serve_static_file(uri: Uri) -> Result<Response<Body>, StatusCode> {
    let path = uri.path().trim_start_matches('/');

    let path = if path.is_empty() { "index.html" } else { path };

    let file = WEB_UI_DIR
        .get_file(path)
        .or_else(|| WEB_UI_DIR.get_file("index.html")) // SPA fallback
        .ok_or(StatusCode::NOT_FOUND)?;

    let content_type = from_path(path).first_or_octet_stream();
    let body = Body::from(file.contents());
    let mut response = Response::new(body);

    if let Ok(header_value) = HeaderValue::from_str(content_type.as_ref()) {
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, header_value);
    }

    Ok(response)
}
