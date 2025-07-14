use std::{ffi::OsString, path::PathBuf, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Result,
};
use axum_extra::extract::CookieJar;
use sf_core::{FileAccess, PreviewData};

use crate::{
    AppState, db,
    error::{SimplyError, err},
    protected::standalone_auth,
};

pub const PREVIEW_FILE_LIMIT: i64 = 512_000_000; // 512 MB

pub async fn get_preview_data(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<PreviewData>, SimplyError> {
    let file = match db::file::get_via_id(&state.db, &id).await {
        Ok(f) => f,
        Err(err) => match err {
            sqlx::Error::RowNotFound => err!("No file with this id found", NOT_FOUND),
            _ => return Err(SimplyError::from(err)),
        },
    };

    if !state.fs.exists(&file.path).await.unwrap_or(false) {
        err!("No actual file found", NOT_FOUND);
    }

    if file.get_access() == FileAccess::Private
        && !standalone_auth(&jar, &headers, &state.config.token)
    {
        err!("You can't access this file", UNAUTHORIZED);
    }

    let data = PreviewData {
        size: file.size,
        file_name: PathBuf::from(&file.path)
            .file_name()
            .unwrap_or(&OsString::from("unknown name"))
            .to_string_lossy()
            .to_string(),
        id,
        created_at: file.created_at.clone(),
        mime_type: mime_guess::from_path(&file.path)
            .first()
            .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM)
            .to_string(),
        access: file.get_access() as i64,
        // only send the path if its an authorized user no matter
        path: if standalone_auth(&jar, &headers, &state.config.token) {
            Some(file.path)
        } else {
            None
        },
        cant_preview: file.size > PREVIEW_FILE_LIMIT,
    };

    Ok(Json(data))
}
