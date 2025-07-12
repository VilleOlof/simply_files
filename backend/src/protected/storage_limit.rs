use std::sync::Arc;

use axum::{Json, extract::State, response::Result};
use serde::Serialize;

use crate::{AppState, db, error::SimplyError};

#[derive(Debug, Serialize)]
pub struct StorageLimit {
    used: u64,
    max: u64,
}

pub async fn get_used_storage_space(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StorageLimit>, SimplyError> {
    let max_bytes = state.config.storage_limit as u64;
    let used_bytes = db::file::get_bytes_stored(&state.db).await?;

    Ok(Json(StorageLimit {
        used: used_bytes,
        max: max_bytes,
    }))
}
