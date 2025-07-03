use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Result,
};

use crate::{AppState, db::file::File};

pub async fn path_to_id(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<File>, StatusCode> {
    let file = File::get_via_path(&state.db, &path).await.unwrap();
    Ok(Json(file))
}
