use std::{path::PathBuf, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Result,
};
use sf_core::ClientFile;

use crate::{AppState, error::SimplyError};

pub async fn get_files(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    get(state, Some(&path)).await
}

pub async fn get_root(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    get(state, None).await
}

async fn get(
    state: Arc<AppState>,
    path: Option<&str>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    let files = state.fs.list_dir(path.unwrap_or("")).await?;

    let db_files = crate::db::file::get_files_in_directory(&state.db, &path.unwrap_or("")).await?;

    let files = ClientFile::from(PathBuf::from(path.unwrap_or("")), files, db_files);

    let files = files
        .iter()
        // hide .public_uploads directory
        .filter(|f| !f.path.starts_with(".public_uploads"))
        .map(|f| f.clone())
        .collect();

    Ok(Json(files))
}

pub async fn add_directory(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    state.fs.create_dir_all(&path).await?;
    Ok(StatusCode::OK)
}

pub async fn delete_directory(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    state.fs.delete_empty_dir(&path).await?;
    Ok(StatusCode::OK)
}
