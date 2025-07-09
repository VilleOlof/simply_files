//! "Public" upload refers to files uploaded via one-time links that anyone with it can use

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::{ConnectInfo, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppState,
    db::links::FileLink,
    generate_id,
    upload::websocket::{self, WebsocketData},
};

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    id: String,
}

// https://simply-backend.lifelike.dev/o/upload/2025-05-23%2024-52.mkv?id=bo4WvY1JKl
#[axum::debug_handler]
pub async fn upload(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UploadQuery>,
    axum::extract::Path(name): axum::extract::Path<String>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    tracing::trace!("Starting public file upload");
    let link = match FileLink::get_via_id(&state.db, &query.id).await {
        Ok(l) => l,
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, "No link with this id found").into_response();
            }
            _ => {
                tracing::error!("{err:?}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch link in DB",
                )
                    .into_response();
            }
        },
    };

    tracing::trace!("Validating link");
    if !link.is_valid_to_use() {
        return (StatusCode::UNAUTHORIZED, "Invalid link").into_response();
    }

    let name = match PathBuf::from(name).file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return (StatusCode::BAD_REQUEST, "Invalid file name").into_response(),
    };

    let linked_path = PathBuf::from(".public_uploads").join(&name);

    let id = generate_id(None);
    websocket::upload_handler(
        ws,
        WebsocketData {
            state: state.clone(),
            id: id.clone(),
            addr,
            path: linked_path.to_string_lossy().to_string(),
            link: Some(link),
        },
    )
    .await
}
