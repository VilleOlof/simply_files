use std::{io::Cursor, sync::Arc};

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use image::{ImageFormat, Luma};
use qrcode::QrCode;

use crate::{
    AppState,
    db::links::FileLink,
    error::{SimplyError, err},
};

pub async fn new_link(State(state): State<Arc<AppState>>) -> Result<Response, SimplyError> {
    let link = FileLink::new(&state.db).await?;
    Ok(Json(link).into_response())
}

pub async fn get_unused_links(State(state): State<Arc<AppState>>) -> Result<Response, SimplyError> {
    let links = FileLink::get_unused_links(&state.db).await?;
    Ok(Json(links).into_response())
}

pub async fn delete_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, SimplyError> {
    FileLink::delete(&state.db, &id).await?;
    Ok(StatusCode::OK)
}

// TODO: this isnt a "protected" route so move links alltogether to a different module
pub async fn verify_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, SimplyError> {
    match FileLink::get_via_id(&state.db, &id).await {
        Ok(l) => {
            if l.is_valid_to_use() {
                Ok(StatusCode::OK)
            } else {
                err!("Link is no longer valid", BAD_REQUEST)
            }
        }
        Err(err) => Err(SimplyError::from(err)),
    }
}

pub async fn qr_code(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, SimplyError> {
    let web_url = match &state.config.web_url {
        Some(u) => u,
        None => {
            err!(
                "No web_url provided in config, unable to create QRCode for link",
                INTERNAL_SERVER_ERROR
            );
        }
    };

    let link = FileLink::get_via_id(&state.db, &id).await?;
    if !link.is_valid_to_use() {
        err!("Link is no longer valid", BAD_REQUEST);
    }

    let code = QrCode::new(format!("{}/u/{}", web_url, link.id))?;
    let mut image_bytes: Vec<u8> = Vec::new();
    code.render::<Luma<u8>>()
        .build()
        .write_to(&mut Cursor::new(&mut image_bytes), ImageFormat::Png)?;

    Ok(Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .body(Body::from(image_bytes))?)
}
