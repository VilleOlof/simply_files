use std::{ffi::OsString, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{self, CONTENT_DISPOSITION, TRANSFER_ENCODING},
    },
    response::{Response, Result},
};
use axum_extra::extract::CookieJar;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::{
    AppState,
    db::file::{File, FileAccess},
    download_stream::DownloadStream,
    error::{SimplyError, err},
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
) -> Result<Response<DownloadStream>, SimplyError> {
    let file = match File::get_via_id(&state.db, &id).await {
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
        && !standalone_auth(jar, headers, &state.config.token)
    {
        err!("You can't access this file", UNAUTHORIZED);
    }

    let body = match state.fs.read_stream(&file.path).await {
        Ok(s) => DownloadStream::new(s, file.id.clone(), state.clone()),
        Err(err) => return Err(SimplyError::from(err)),
    };

    let mut res = Response::builder()
        .header(CONTENT_DISPOSITION, content_disposition(&file.path))
        .header(TRANSFER_ENCODING, HeaderValue::from_static("chunked"))
        .body(body)?;

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
    .unwrap_or(HeaderValue::from_static(
        "attachment; filename*=UTF-8''unknown",
    ))
}
