use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;

use crate::{AppState, upload::private};

mod authenticate;
pub mod link;
mod logout;

/// Routes that require a valid "token" specified in the config as Authorization header
pub fn protected_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/check", get(|| async { "Simply... Files" }))
        .route("/logout", get(logout::logout))
        .route("/upload/{*path}", post(private::upload))
        .route("/new_link", post(link::new_link))
        .route_layer(from_fn_with_state(state.clone(), token_auth))
        .route("/authenticate", post(authenticate::authenticate))
        .with_state(state.clone())
}

/// Main authentication layer for these routes
async fn token_auth(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match standalone_auth(jar, headers, &state.config.token) {
        true => (),
        false => return Err(StatusCode::UNAUTHORIZED),
    };

    let response = next.run(request).await;
    Ok(response)
}

/// token can be given via either a cookie (token=<token>)
/// or via headers (Authorization = Bearer <token>)
pub fn standalone_auth(jar: CookieJar, headers: HeaderMap, actual_token: &str) -> bool {
    let token = match jar.get("token") {
        Some(t) => t.value(),
        None => match headers.get("Authorization") {
            Some(h) => match h.to_str() {
                Ok(h) => h.trim_start_matches("Bearer "),
                Err(_) => return false,
            },
            None => return false,
        },
    }
    .trim();

    if token != actual_token {
        return false;
    }

    true
}
