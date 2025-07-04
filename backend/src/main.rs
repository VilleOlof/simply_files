use std::{
    fs::{File, exists},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use sqlx::{SqlitePool, pool::PoolOptions};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    config::Config, file_system::FileSystem, protected::protected_routes, speed_test::speed_test,
};

mod config;
mod db;
mod download;
mod download_stream;
mod error;
mod file_system;
mod protected;
mod speed_test;
mod upload;

#[derive(Debug)]
pub struct AppState {
    config: Config,
    fs: Box<dyn FileSystem>,
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let config = Config::read_config();
    let fs = config.get_file_system();

    if !exists(&config.db).expect("Failed to check if the database file exists") {
        File::create(&config.db).expect("Failed to create missing database file");
    }

    if let Err(err) = init_folders(&fs).await {
        panic!("Failed to create base folders, can't continue: {err:?}");
    }

    let db = PoolOptions::new().connect(&config.db).await.unwrap();
    db::init(&db).await.expect("Failed to init database tables");

    let addr = config.addr.clone(); // just so it lives long enough
    let (upload_limit, upload_timeout) = (config.upload_limit, config.upload_timeout);
    let state = Arc::new(AppState { config, fs, db });

    let app = Router::new()
        .route("/", get(root))
        .route("/d/{*id}", get(download::download))
        .route("/o/upload/{*name}", post(upload::public::upload))
        .route("/verify_link/{*id}", post(protected::link::verify_link))
        .with_state(state.clone())
        .nest("/speed_test", speed_test())
        .nest("/m", protected_routes(state.clone()))
        .layer(CorsLayer::very_permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(upload_timeout)))
        .layer(DefaultBodyLimit::max(upload_limit));

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

/// Generates a random id with A-Z, a-z & 0-9
pub fn generate_id(len: Option<usize>) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890";

    let len = len.unwrap_or(10);

    let mut rng = rand::rng();
    let hash: String = (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    hash
}

async fn init_folders(fs: &Box<dyn FileSystem>) -> std::io::Result<()> {
    let root = fs.root_directory().await.to_string_lossy().to_string();
    if !fs.exists(&root).await? {
        fs.create_dir_all(&root).await?;
    }
    let public_uploads = PathBuf::from(root)
        .join(".public_uploads")
        .to_string_lossy()
        .to_string();
    if !fs.exists(&public_uploads).await? {
        fs.create_dir_all(&public_uploads).await?;
    }

    Ok(())
}
