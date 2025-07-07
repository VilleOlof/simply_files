use axum::Json;
use axum::body::BodyDataStream;
use axum::response::IntoResponse;
use axum::{body::Body, http::StatusCode, response::Response};
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::db::file::File;
use crate::error::{SimplyError, err};
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

    tracing::trace!("Checking if file path is a valid one");
    // all uploads pass through here so we can validate shit here
    if !path_is_valid(&path) {
        tracing::error!("{:?} is invalid", path.as_ref());
        return Err(Error::new(ErrorKind::Other, "Path is invalid"));
    }

    tracing::trace!("Mapping body_stream into byte_stream");
    let byte_stream = stream.map(|frame_result| {
        frame_result
            .map(|frame| frame.to_vec())
            .map_err(|e| Error::new(ErrorKind::Other, e))
    });
    let pinned_stream: FSStream = Box::pin(byte_stream);

    tracing::trace!("Starting stream write to file_system");
    fs.write_stream(&path.as_ref().to_string_lossy(), pinned_stream)
        .await
}

#[tracing::instrument(skip(state, body))]
pub async fn handler_upload(
    state: &Arc<AppState>,
    path: &str,
    id: &str,
    body: Body,
) -> Result<Response, SimplyError> {
    tracing::trace!("Checking if theres available storage");
    let bytes_stored = File::get_bytes_stored(&state.db).await?;
    if bytes_stored > state.config.storage_limit as u64 {
        err!("Storage limit reached", INSUFFICIENT_STORAGE);
    }

    tracing::trace!("Creating new file entry in DB");
    let mut file = match File::new(&state.db, &id, &path).await {
        Err(err) => {
            clean_up(&state, &id, &path).await?;
            err!("Failed to create file entry", INTERNAL_SERVER_ERROR, err);
        }
        Ok(f) => f,
    };

    tracing::trace!("Convert body into data stream");
    let data_stream = body.into_data_stream();
    match upload_via_stream(&state.fs, data_stream, &path).await {
        Ok(_) => (),
        Err(err) => {
            // if the upload_stream fails, we need to backtrack to not get loose files
            clean_up(&state, &id, &path).await?;
            err!("Failed upload via streaming", INTERNAL_SERVER_ERROR, err);
        }
    };

    let metadata = state.fs.metadata(&path).await?;
    file.successful_upload(&state.db, metadata.size as i64)
        .await?;

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

async fn clean_up(state: &Arc<AppState>, id: &str, path: &str) -> Result<(), SimplyError> {
    File::delete(&state.db, &id).await?;

    if state.fs.exists(&path).await? {
        state.fs.delete(&path).await?;
    }

    Ok(())
}
