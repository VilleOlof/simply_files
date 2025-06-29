use std::sync::Arc;

use axum::{Router, routing::get};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{config::Config, file_system::FileSystem};

mod config;
mod file_system;

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

    let port = config.port; // just so it lives long enough
    let state = Arc::new(AppState { config, fs });

    let app = Router::new().route("/", get(root)).with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect(&format!("Failed to bind to 0.0.0.0:{}", port));
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
