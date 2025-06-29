use std::{pin::Pin, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use tokio_stream::{Stream, StreamExt};

use crate::AppState;

mod authenticate;
mod logout;

/// Routes that require a valid "token" specified in the config as Authorization header
pub fn protected_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/check", get(|| async { "Simply... Files" }))
        .route("/logout", get(logout::logout))
        .route("/upload/{*path}", post(stream_upload))
        .route_layer(from_fn_with_state(state.clone(), token_auth))
        .route("/authenticate", post(authenticate::authenticate))
        .with_state(state.clone())
}

async fn stream_upload(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: Body,
) -> Result<Response, StatusCode> {
    let data_stream = body.into_data_stream();

    let byte_stream = data_stream.map(|frame_result| {
        frame_result
            .map(|frame| frame.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    });
    let pinned_stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>, std::io::Error>> + Send>> =
        Box::pin(byte_stream);

    match state.fs.write_stream(&path, pinned_stream).await {
        Ok(_) => Ok(StatusCode::CREATED.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Main authentication layer
///
/// token can be given via either a cookie (token=<token>)
/// or via headers (Authorization = Bearer <token>)
async fn token_auth(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = match jar.get("token") {
        Some(t) => t.value(),
        None => match headers.get("Authorization") {
            Some(h) => match h.to_str() {
                Ok(h) => h.trim_start_matches("Bearer "),
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            },
            None => return Err(StatusCode::UNAUTHORIZED),
        },
    }
    .trim();

    if token != state.config.token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let response = next.run(request).await;
    Ok(response)
}
