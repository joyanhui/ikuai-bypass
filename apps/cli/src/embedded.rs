use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontends/app/dist"]
pub struct FrontendDist;

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}

pub async fn handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match FrontendDist::get(path) {
        Some(content) => (
            [(header::CONTENT_TYPE, mime_type(path))],
            content.data.into_owned(),
        )
            .into_response(),
        None => {
            // SPA fallback: serve index.html for client-side routing
            match FrontendDist::get("index.html") {
                Some(content) => (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    content.data.into_owned(),
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "Not found").into_response(),
            }
        }
    }
}
