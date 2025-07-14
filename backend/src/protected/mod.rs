use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::HeaderMap,
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::{any, delete, get, post},
};
use axum_extra::extract::CookieJar;

use crate::{
    AppState,
    error::{SimplyError, err},
    upload::private,
};

mod authenticate;
mod directory;
mod file;
mod file_system;
pub mod link;
mod logout;
pub mod path_to_id;
mod storage_limit;

/// Routes that require a valid "token" specified in the config as Authorization header
pub fn protected_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/check", get(|| async { "Simply... Files" }))
        .route("/logout", get(logout::logout))
        .route("/upload/{*path}", any(private::upload))
        .route("/new_link", post(link::new_link))
        .route("/links", get(link::get_unused_links))
        .route("/link/{*id}", delete(link::delete_link))
        .route("/file_system", get(file_system::get_file_system))
        .route("/storage_limit", get(storage_limit::get_used_storage_space))
        .route("/directory/{*path}", get(directory::get_files))
        .route("/directory", get(directory::get_root))
        .route("/directory/{*path}", post(directory::add_directory))
        .route("/directory/{*path}", delete(directory::delete_directory))
        .route("/delete_file/{*path}", delete(file::remove_file))
        .route("/rename_file/{*path}", post(file::rename_file))
        .route("/access/{*path}", post(file::change_access))
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
) -> Result<Response, SimplyError> {
    match standalone_auth(&jar, &headers, &state.config.token) {
        true => (),
        false => err!("Invalid token", UNAUTHORIZED),
    };

    let response = next.run(request).await;
    Ok(response)
}

/// token can be given via either a cookie (token=<token>)
/// or via headers (Authorization = Bearer <token>)
pub fn standalone_auth(jar: &CookieJar, headers: &HeaderMap, actual_token: &str) -> bool {
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
