use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    AppState,
    db::links::FileLink,
    error::{SimplyError, err},
};

pub async fn new_link(State(state): State<Arc<AppState>>) -> Result<Response, SimplyError> {
    let link = FileLink::new(&state.db).await?;
    Ok(Json(link).into_response())
}

pub async fn get_unused_links(State(state): State<Arc<AppState>>) -> Result<Response, SimplyError> {
    let links = FileLink::get_unused_links(&state.db).await?;
    Ok(Json(links).into_response())
}

pub async fn delete_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, SimplyError> {
    FileLink::delete(&state.db, &id).await?;
    Ok(StatusCode::OK)
}

// TODO: this isnt a "protected" route so move links alltogether to a different module
pub async fn verify_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, SimplyError> {
    match FileLink::get_via_id(&state.db, &id).await {
        Ok(l) => {
            if l.is_valid_to_use() {
                Ok(StatusCode::OK)
            } else {
                err!("Link is no longer valid", BAD_REQUEST)
            }
        }
        Err(err) => Err(SimplyError::from(err)),
    }
}
