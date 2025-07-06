use axum::{
    Router,
    body::{Body, Bytes},
    http::{Response, StatusCode},
    response::{IntoResponse, Result},
    routing::{get, post},
};

use crate::error::SimplyError;

const DOWNLOAD_SIZE: usize = 100 * 1024 * 1024; // 100 MB
async fn download() -> Result<Response<Body>, SimplyError> {
    let data = vec![0u8; DOWNLOAD_SIZE];
    Ok(Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(data))?)
}

async fn upload(_payload: Bytes) -> impl IntoResponse {
    StatusCode::OK
}

pub fn speed_test() -> Router {
    Router::new()
        .route("/download", get(download))
        .route("/upload", post(upload))
}
