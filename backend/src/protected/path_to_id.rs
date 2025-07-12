use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::Result,
};
use sf_core::File;

use crate::{AppState, db, error::SimplyError};

pub async fn path_to_id(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<File>, SimplyError> {
    let file = db::file::get_via_path(&state.db, &path).await?;
    Ok(Json(file))
}
