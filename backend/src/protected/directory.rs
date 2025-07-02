use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Result,
};

use crate::{AppState, file_system::FileMetadata};

pub async fn get_files(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileMetadata>>, StatusCode> {
    get(state, Some(&path)).await
}

pub async fn get_root(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileMetadata>>, StatusCode> {
    get(state, None).await
}

async fn get(
    state: Arc<AppState>,
    path: Option<&str>,
) -> Result<Json<Vec<FileMetadata>>, StatusCode> {
    let files = match state.fs.list_dir(path.unwrap_or("")).await {
        Ok(f) => f,
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let files = files
        .iter()
        // hide .public_uploads directory
        .filter(|f| !f.path.starts_with(".public_uploads"))
        .map(|f| f.clone())
        .collect();

    Ok(Json(files))
}
