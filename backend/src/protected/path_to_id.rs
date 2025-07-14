use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Result,
};
use axum_extra::extract::CookieJar;
use sf_core::{File, FileAccess};

use crate::{
    AppState, db,
    error::{SimplyError, err},
    protected::standalone_auth,
};

pub async fn path_to_id(
    jar: CookieJar,
    headers: HeaderMap,
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<File>, SimplyError> {
    let file = db::file::get_via_path(&state.db, &path).await?;

    if file.get_access() == FileAccess::Private
        && !standalone_auth(&jar, &headers, &state.config.token)
    {
        err!("You can't access this file", UNAUTHORIZED);
    }

    Ok(Json(file))
}

pub async fn id_to_path(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<File>, SimplyError> {
    let file = db::file::get_via_id(&state.db, &id).await?;

    if file.get_access() == FileAccess::Private
        && !standalone_auth(&jar, &headers, &state.config.token)
    {
        err!("You can't access this file", UNAUTHORIZED);
    }

    Ok(Json(file))
}
