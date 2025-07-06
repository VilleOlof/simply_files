use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Result,
};
use serde::Deserialize;

use crate::{
    AppState,
    db::file::{File, FileAccess},
    error::SimplyError,
};

pub async fn remove_file(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    let db_file = File::get_via_path(&state.db, &path).await?;

    File::delete(&state.db, &db_file.id).await?;
    state.fs.delete(&path).await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct RenameQuery {
    pub to: String,
}

pub async fn rename_file(
    Path(path): Path<String>,
    Query(query): Query<RenameQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    let mut db_file = File::get_via_path(&state.db, &path).await?;

    db_file.rename(&state.db, &query.to).await?;
    state.fs.rename(&path, &query.to).await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct ChangeAccessQuery {
    pub access: i64,
    pub id: Option<bool>,
}

pub async fn change_access(
    Path(path): Path<String>,
    Query(query): Query<ChangeAccessQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    let access: FileAccess = query.access.into();

    let mut file = if query.id.unwrap_or(false) {
        File::get_via_id(&state.db, &path).await?
    } else {
        File::get_via_path(&state.db, &path).await?
    };

    file.change_access(&state.db, access).await?;

    Ok(StatusCode::OK)
}
