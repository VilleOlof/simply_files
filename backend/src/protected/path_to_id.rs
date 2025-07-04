use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::Result,
};

use crate::{AppState, db::file::File, error::SimplyError};

pub async fn path_to_id(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<File>, SimplyError> {
    let file = File::get_via_path(&state.db, &path).await?;
    Ok(Json(file))
}
