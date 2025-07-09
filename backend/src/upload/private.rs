use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
};
use std::{net::SocketAddr, sync::Arc};

use crate::{
    AppState, generate_id,
    upload::websocket::{self, WebsocketData},
};

// https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv
pub async fn upload(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let id = generate_id(None);
    websocket::upload_handler(
        ws,
        WebsocketData {
            state,
            addr,
            id,
            path,
            link: None,
        },
    )
    .await
}
