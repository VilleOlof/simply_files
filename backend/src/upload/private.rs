use axum::{
    Json,
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{AppState, db::file::File, generate_id, upload::upload_via_stream};

// https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv
pub async fn upload(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: Body,
) -> Result<Response, StatusCode> {
    let id = generate_id(None);
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

async fn clean_up(state: Arc<AppState>, id: &str, path: &str) {
    // TODO: remove these unwraps
    File::delete(&state.db, &id).await.unwrap();

    if state.fs.exists(&path).await.unwrap() {
        state.fs.delete(&path).await.unwrap();
    }
}
