use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::AppState;

pub async fn path_to_id(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // remove root from the path FileMetadata
    // and then where = path and use that id
    ""
}
