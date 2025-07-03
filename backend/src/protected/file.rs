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
};

pub async fn remove_file(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    let db_file = File::get_via_path(&state.db, &path).await.unwrap();
    File::delete(&state.db, &db_file.id).await.unwrap();

    match state.fs.delete(&path).await {
        Err(err) => {
            tracing::error!("{err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(_) => Ok(StatusCode::OK),
    }
}

#[derive(Debug, Deserialize)]
pub struct RenameQuery {
    pub new: String,
}

pub async fn rename_file(
    Path(path): Path<String>,
    Query(query): Query<RenameQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    todo!("Sync with db");
    match state.fs.rename(&path, &query.new).await {
        Err(err) => {
            tracing::error!("{err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(_) => Ok(StatusCode::OK),
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangeAccessQuery {
    pub access: i64,
}

pub async fn change_access(
    Path(path): Path<String>,
    Query(query): Query<ChangeAccessQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    let access: FileAccess = query.access.into();
    let mut file = File::get_via_path(&state.db, &path).await.unwrap();

    file.change_access(&state.db, access).await.unwrap();

    Ok(StatusCode::OK)
}
