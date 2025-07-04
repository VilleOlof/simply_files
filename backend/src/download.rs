use std::{ffi::OsString, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{self, CONTENT_DISPOSITION, TRANSFER_ENCODING},
    },
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::{
    AppState,
    db::file::{File, FileAccess},
    download_stream::DownloadStream,
    protected::standalone_auth,
};

/// The one and only: Download
/// Any download EVER is through this handler
///
/// This checks if the file is private, public
/// If the request has access to the specified file
/// If it exists. And if so streams it properly to the client
/// With content_disposition & mime_type
pub async fn download(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let file = match File::get_via_id(&state.db, &id).await {
        Ok(f) => f,
        Err(err) => match err {
            sqlx::Error::RowNotFound => return Err(StatusCode::NOT_FOUND),
            _ => {
                tracing::error!("{err:?}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    if !state.fs.exists(&file.path).await.unwrap_or(false) {
        tracing::error!("No file at path: {:?}", file.path);
        return Err(StatusCode::NOT_FOUND);
    }

    if file.get_access() == FileAccess::Private
        && !standalone_auth(jar, headers, &state.config.token)
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let body = match state.fs.read_stream(&file.path).await {
        Ok(s) => DownloadStream::new(s, file.id.clone(), state.clone()),
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut res = match Response::builder()
        .header(CONTENT_DISPOSITION, content_disposition(&file.path))
        .header(TRANSFER_ENCODING, HeaderValue::from_static("chunked"))
        .body(body)
    {
        Ok(r) => r,
        Err(err) => {
            tracing::error!("{err:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Some(mime) = get_mime_type(&file.path) {
        res.headers_mut().insert(header::CONTENT_TYPE, mime);
    }

    Ok(res)
}

fn get_mime_type(path: &str) -> Option<HeaderValue> {
    match mime_guess::from_path(&path).first() {
        Some(mt) => match mt.to_string().parse::<HeaderValue>() {
            Ok(mime_value) => Some(mime_value),
            Err(_) => None,
        },
        None => None,
    }
}

fn content_disposition(path: &str) -> HeaderValue {
    let path = PathBuf::from(path);
    let name = path
        .file_name()
        .unwrap_or(&OsString::from("unknown"))
        .to_string_lossy()
        .to_string();

    HeaderValue::from_str(&format!(
        "attachment; filename*=UTF-8''{}",
        utf8_percent_encode(&name, NON_ALPHANUMERIC).to_string()
    ))
    .unwrap()
}
