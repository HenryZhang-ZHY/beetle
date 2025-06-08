# Web UI Embedding Design Document

## Overview

This document outlines the design for embedding the React-based Web UI (`apps/webui`) directly into the CLI binary (`apps/cli`) so that users can access the web interface when running `beetle serve` without requiring separate web UI deployment.

## Problem Statement

Currently, users need to:
1. Run `beetle serve` to start the API server
2. Separately run the web UI development server or deploy it independently

This creates friction for users who want a complete, integrated experience out of the box.

## Solution Overview

Embed the pre-built web UI static files directly into the CLI binary using Rust's `include_dir!` macro, and serve them via the existing Axum HTTP server alongside the API endpoints.

## Simple Implementation

### Step 1: Update CLI Dependencies

Add to `apps/cli/Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies ...
include_dir = "0.7"
mime_guess = "2.0"

[build-dependencies]
anyhow = "1.0"
```

### Step 2: Create Build Script

Create `apps/cli/build.rs`:
```rust
use std::process::Command;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only build web UI for release builds
    if std::env::var("PROFILE")? == "release" {
        build_web_ui()?;
    }
    Ok(())
}

fn build_web_ui() -> Result<(), Box<dyn std::error::Error>> {
    let webui_dir = Path::new("../../apps/webui");
    
    if !webui_dir.exists() {
        println!("cargo:warning=Web UI directory not found, skipping embed");
        return Ok(());
    }
    
    // Build the web UI
    let status = Command::new("bun")
        .current_dir(webui_dir)
        .args(&["run", "build"])
        .status()?;

    if !status.success() {
        return Err("Failed to build web UI".into());
    }

    println!("cargo:warning=Web UI built successfully");
    Ok(())
}
```

### Step 3: Add Static File Serving

Create `apps/cli/src/static_files.rs`:
```rust
use axum::response::Response;
use axum::http::{header, StatusCode, HeaderValue, Uri};
use axum::body::Body;
use include_dir::{Dir, include_dir};
use mime_guess::from_path;

static WEB_UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../apps/webui/dist");

pub async fn serve_static_file(uri: Uri) -> Result<Response<Body>, StatusCode> {
    let path = uri.path().trim_start_matches('/');
    
    let path = if path.is_empty() {
        "index.html"
    } else {
        path
    };

    let file = WEB_UI_DIR.get_file(path)
        .or_else(|| WEB_UI_DIR.get_file("index.html"))  // SPA fallback
        .ok_or(StatusCode::NOT_FOUND)?;

    let content_type = from_path(path).first_or_octet_stream();
    let body = Body::from(file.contents());
    let mut response = Response::new(body);
    
    if let Ok(header_value) = HeaderValue::from_str(content_type.as_ref()) {
        response.headers_mut().insert(header::CONTENT_TYPE, header_value);
    }

    Ok(response)
}
```

### Step 4: Update Server

Add to `apps/cli/src/lib.rs`:
```rust
pub mod static_files;
```

Update `apps/cli/src/server.rs`:
```rust
use crate::static_files::serve_static_file;

// In HttpServer::start method, add fallback route:
let app = Router::new()
    .route("/api/indexes", get(list_indexes).post(create_index))
    .route("/api/indexes/{index_name}", get(get_index_details).delete(delete_index))
    .route("/api/indexes/{index_name}/search", get(search_index))
    .route("/api/indexes/{index_name}/reindex", post(reindex_index))
    .route("/api/indexes/{index_name}/update", post(update_index))
    .fallback(serve_static_file);
```

### Step 5: Update Web UI Build

Update `apps/webui/vite.config.ts`:
```typescript
export default defineConfig({
  base: './',  // Use relative paths
  // ... rest of existing config
})
```

### Step 6: Build

Build the CLI with embedded web UI:
```bash
cargo build --release
```

The web UI will be accessible at `http://localhost:3000` when running `beetle serve --port 3000`.

## Testing

Build and test:
```bash
# Build the web UI and CLI together
cargo build --release

# Test it works
./target/release/beetle serve --port 3000
# Navigate to http://localhost:3000
```

## Notes

- Web UI is only built during release builds to avoid slowing down development
- The web UI will be served at the same port as the API
- API endpoints are available at `/api/*` 
- All other routes serve the embedded web UI
- The binary will be about 2-3MB larger due to embedded assets
