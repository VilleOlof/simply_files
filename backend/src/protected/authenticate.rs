use std::{sync::Arc, time::Duration};

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthenticateBody {
    token: String,
}

pub async fn authenticate(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    body: Json<AuthenticateBody>,
) -> Result<CookieJar, StatusCode> {
    if body.token != state.config.token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut expire = OffsetDateTime::now_utc();
    expire += Duration::from_secs(15_768_000);

    let mut cookie = Cookie::new("token", body.token.clone());
    cookie.set_expires(expire);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::None);
    cookie.set_http_only(true);

    Ok(jar.add(cookie))
}
