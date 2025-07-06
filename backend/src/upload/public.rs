//! "Public" upload refers to files uploaded via one-time links that anyone with it can use

use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::Response,
};
use serde::Deserialize;

use crate::{
    AppState,
    db::{
        file::{File, FileAccess},
        links::FileLink,
    },
    error::{SimplyError, err},
    generate_id,
    upload::handler_upload,
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
    body: Body,
) -> Result<Response, SimplyError> {
    let link = match FileLink::get_via_id(&state.db, &query.id).await {
        Ok(l) => l,
        Err(err) => match err {
            sqlx::Error::RowNotFound => err!("No link with this id found", NOT_FOUND),
            _ => return Err(SimplyError::from(err)),
        },
    };

    if !link.is_valid_to_use() {
        err!("Invalid link", UNAUTHORIZED);
    }

    let name = match PathBuf::from(name).file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => err!("Invalid file name", BAD_REQUEST),
    };

    let linked_path = PathBuf::from(".public_uploads").join(&name);

    let id = generate_id(None);

    let response = handler_upload(
        &state,
        &linked_path.to_string_lossy().to_string(),
        &id,
        body,
    )
    .await?;

    // sucess
    link.uploaded_with(&state.db, &id).await?;
    // always change one-time "public" uploads to well, Public
    let mut file = File::get_via_id(&state.db, &id).await?;
    file.change_access(&state.db, FileAccess::Public).await?;

    Ok(response)
}
