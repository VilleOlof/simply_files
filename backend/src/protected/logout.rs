use std::sync::Arc;

use axum::extract::State;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use crate::AppState;

pub async fn logout(jar: CookieJar, State(state): State<Arc<AppState>>) -> CookieJar {
    let mut cookie = Cookie::new("token", "");
    cookie.set_path("/");
    cookie.set_secure(true);
    if let Some(domain) = &state.config.cookie_domain {
        cookie.set_domain(domain.clone());
    }
    cookie.set_same_site(SameSite::None);
    cookie.set_http_only(true);

    jar.remove(cookie)
}
