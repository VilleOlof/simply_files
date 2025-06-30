use axum::{
    body::{Body, BodyDataStream},
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{path::Path, sync::Arc};
use tokio_stream::StreamExt;

use crate::{
    AppState,
    file_system::{FileSystem, WriteStream},
};

/// Main function for streaming files from a client to the given file system
async fn upload_via_stream(
    fs: &Box<dyn FileSystem>,
    stream: BodyDataStream,
    path: impl AsRef<Path>,
) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};

    // all uploads pass through here so we can validate shit here
    if !path_is_valid(&path) {
        return Err(Error::new(ErrorKind::Other, "Path is invalid"));
    }

    let byte_stream = stream.map(|frame_result| {
        frame_result
            .map(|frame| frame.to_vec())
            .map_err(|e| Error::new(ErrorKind::Other, e))
    });
    let pinned_stream: WriteStream = Box::pin(byte_stream);

    fs.write_stream(&path.as_ref().to_string_lossy(), pinned_stream)
        .await
}

/// A path cannot be root or go back or anything foul
fn path_is_valid(path: impl AsRef<std::path::Path>) -> bool {
    let mut components = path.as_ref().components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

pub async fn private_upload(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: Body,
) -> Result<Response, StatusCode> {
    let data_stream = body.into_data_stream();
    match upload_via_stream(&state.fs, data_stream, path).await {
        Ok(_) => Ok(StatusCode::CREATED.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
// https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv
