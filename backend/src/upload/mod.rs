use axum::Json;
use axum::body::BodyDataStream;
use axum::response::IntoResponse;
use axum::{body::Body, http::StatusCode, response::Response};
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::db::file::File;
use crate::{
    AppState,
    file_system::{FSStream, FileSystem},
};

pub mod private;
pub mod public;

/// Main function for streaming files from a client to the given file system
async fn upload_via_stream(
    fs: &Box<dyn FileSystem>,
    stream: BodyDataStream,
    path: impl AsRef<Path>,
) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};

    // all uploads pass through here so we can validate shit here
    if !path_is_valid(&path) {
        tracing::error!("{:?} is invalid", path.as_ref());
        return Err(Error::new(ErrorKind::Other, "Path is invalid"));
    }

    let byte_stream = stream.map(|frame_result| {
        frame_result
            .map(|frame| frame.to_vec())
            .map_err(|e| Error::new(ErrorKind::Other, e))
    });
    let pinned_stream: FSStream = Box::pin(byte_stream);

    fs.write_stream(&path.as_ref().to_string_lossy(), pinned_stream)
        .await
}

pub async fn handler_upload(
    state: Arc<AppState>,
    path: &str,
    id: &str,
    body: Body,
) -> Result<Response, StatusCode> {
    let mut file = match File::new(&state.db, &id, &path).await {
        Err(err) => {
            tracing::error!("{err:?}");
            clean_up(state, &id, &path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(f) => f,
    };

    let data_stream = body.into_data_stream();
    match upload_via_stream(&state.fs, data_stream, &path).await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{err:?}");
            // if the upload_stream fails, we need to backtrack to not get loose files
            clean_up(state, &id, &path).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let metadata = state.fs.metadata(&path).await.unwrap();
    file.successful_upload(&state.db, metadata.size as i64)
        .await
        .unwrap();

    let mut response = Json(file).into_response();
    *response.status_mut() = StatusCode::CREATED;

    Ok(response)
}

/// A path cannot be root or go back or anything foul
fn path_is_valid(path: impl AsRef<std::path::Path>) -> bool {
    let mut components = path.as_ref().components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    return true;
}

async fn clean_up(state: Arc<AppState>, id: &str, path: &str) {
    // TODO: remove these unwraps
    File::delete(&state.db, &id).await.unwrap();

    if state.fs.exists(&path).await.unwrap() {
        state.fs.delete(&path).await.unwrap();
    }
}
