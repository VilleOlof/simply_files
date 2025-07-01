use axum::{body::Body, extract::State, http::StatusCode, response::Response};
use std::sync::Arc;

use crate::{AppState, generate_id, upload::handler_upload};

// https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv
pub async fn upload(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: Body,
) -> Result<Response, StatusCode> {
    let id = generate_id(None);
    handler_upload(state, &path, &id, body).await
}
