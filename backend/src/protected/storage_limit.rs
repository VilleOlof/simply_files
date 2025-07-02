use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::{AppState, db::file::File};

#[derive(Debug, Serialize)]
struct StorageLimit {
    used: u64,
    max: u64,
}

pub async fn get_used_storage_space(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let max_bytes = state.config.upload_limit as u64;
    let used_bytes = match File::get_bytes_stored(&state.db).await {
        Ok(ub) => ub,
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(StorageLimit {
        used: used_bytes,
        max: max_bytes,
    }))
}
