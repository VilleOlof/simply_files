use std::{sync::Arc, time::Duration};

use axum::{Router, extract::DefaultBodyLimit, routing::get};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    config::Config, file_system::FileSystem, protected::protected_routes, speed_test::speed_test,
};

mod config;
mod file_system;
mod protected;
mod speed_test;

#[derive(Debug)]
pub struct AppState {
    config: Config,
    fs: Box<dyn FileSystem>,
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let config = Config::read_config();
    let fs = config.get_file_system();

    let addr = config.addr.clone(); // just so it lives long enough
    let state = Arc::new(AppState { config, fs });

    let app = Router::new()
        .route("/", get(root))
        .with_state(state.clone())
        .nest("/speed_test", speed_test())
        .nest("/m", protected_routes(state.clone()))
        .layer(CorsLayer::very_permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(86400)))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(&addr).await.expect(&addr);
    axum::serve(listener, app).await.unwrap();
}

fn setup_tracing() {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(sub).expect("Failed setting default subscriber");
}

async fn root() -> &'static str {
    "What will today's adventure be?"
}
