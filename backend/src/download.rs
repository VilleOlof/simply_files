use std::{ffi::OsString, io::Cursor, path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{
        HeaderMap, HeaderValue,
        header::{self, CONTENT_DISPOSITION, CONTENT_TYPE, TRANSFER_ENCODING},
    },
    response::{IntoResponse, Response, Result},
};
use axum_extra::extract::CookieJar;
use image::{ImageFormat, Luma};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use qrcode::QrCode;
use serde::Deserialize;
use sf_core::FileAccess;

use crate::{
    AppState, db,
    download_stream::DownloadStream,
    error::{SimplyError, err},
    preview::PREVIEW_FILE_LIMIT,
    protected::standalone_auth,
};

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    // raw
    r: Option<String>,
    // preview
    p: Option<String>,
}

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
    Query(query): Query<DownloadQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response<DownloadStream>, SimplyError> {
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

    if query.p.unwrap_or(String::from("nuh_uh")) == "t" && file.size > PREVIEW_FILE_LIMIT {
        err!(
            "Can't preview this file, it's above the preview size limit",
            IM_A_TEAPOT
        );
    }

    let body = match state.fs.read_stream(&file.path).await {
        Ok(s) => DownloadStream::new(s, file.id.clone(), state.clone()),
        Err(err) => return Err(SimplyError::from(err)),
    };

    let mut res = Response::builder()
        .header(TRANSFER_ENCODING, HeaderValue::from_static("chunked"))
        .body(body)?;

    if query.r.unwrap_or(String::from("nuh_uh")) != "t" {
        res.headers_mut()
            .insert(CONTENT_DISPOSITION, content_disposition(&file.path));
    }

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

#[derive(Debug, Deserialize)]
pub struct QrCodeQuery {
    pub preview_link: Option<bool>,
}

pub async fn qr_code(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<QrCodeQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, SimplyError> {
    let url = if query.preview_link.unwrap_or(false) {
        match &state.config.web_url {
            Some(u) => u,
            None => {
                err!(
                    "No web_url provided in config, unable to create QRCode for file",
                    INTERNAL_SERVER_ERROR
                );
            }
        }
    } else {
        match &state.config.backend_url {
            Some(u) => u,
            None => {
                err!(
                    "No backend_url provided in config, unable to create QRCode for file",
                    INTERNAL_SERVER_ERROR
                );
            }
        }
    };

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

    if !standalone_auth(&jar, &headers, &state.config.token) {
        err!("You can't access this file", UNAUTHORIZED);
    }

    let code = QrCode::new(format!("{}/d/{}", url, file.id))?;
    let mut image_bytes: Vec<u8> = Vec::new();
    code.render::<Luma<u8>>()
        .build()
        .write_to(&mut Cursor::new(&mut image_bytes), ImageFormat::Png)?;

    Ok(Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .body(Body::from(image_bytes))?)
}
