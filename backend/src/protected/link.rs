use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{AppState, db::links::FileLink};

pub async fn new_link(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let link = match FileLink::new(&state.db).await {
        Ok(l) => l,
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(link).into_response())
}

pub async fn get_unused_links(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let links = match FileLink::get_unused_links(&state.db).await {
        Ok(ls) => ls,
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(links).into_response())
}

pub async fn delete_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match FileLink::delete(&state.db, &id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            tracing::error!("{err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TODO: this isnt a "protected" route so move links alltogether to a different module
pub async fn verify_link(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> StatusCode {
    match FileLink::get_via_id(&state.db, &id).await {
        Ok(l) => {
            if l.is_valid_to_use() {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            }
        }
        Err(err) => {
            tracing::error!("{err:?}");
            StatusCode::BAD_REQUEST
        }
    }
}
