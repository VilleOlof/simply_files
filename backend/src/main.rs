use std::{
    env,
    fs::{File, OpenOptions, exists},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{any, get, post},
};
use sqlx::{SqlitePool, pool::PoolOptions};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer, Registry, layer::SubscriberExt};

use crate::{
    config::Config, file_system::FileSystem, protected::protected_routes, speed_test::speed_test,
};

mod config;
mod db;
mod download;
mod download_stream;
mod error;
mod file_system;
mod preview;
mod protected;
mod speed_test;
mod sync;
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
    let fs = config.get_file_system().await;

    if !exists(&config.db).expect("Failed to check if the database file exists") {
        File::create(&config.db).expect("Failed to create missing database file");
    }

    if let Err(err) = init_folders(&fs).await {
        panic!("Failed to create base folders, can't continue: {err:?}");
    }

    let db = PoolOptions::new()
        .connect(&config.db)
        .await
        .expect("Failed to connect to database");
    db::init(&db).await.expect("Failed to init database tables");

    let addr = config.addr.clone(); // just so it lives long enough
    let (upload_limit, upload_timeout) = (config.upload_limit, config.upload_timeout);
    let state = Arc::new(AppState { config, fs, db });

    if let Err(err) = sync::sync_files(state.clone()).await {
        tracing::error!("Failed syncing database with the file system: {err:?}");
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/d/{*id}", get(download::download))
        .route("/qr/file/{*id}", get(download::qr_code))
        .route("/qr/link/{*id}", get(protected::link::qr_code))
        .route("/preview_data/{*id}", get(preview::get_preview_data))
        .route("/o/upload/{*name}", any(upload::public::upload))
        .route("/verify_link/{*id}", post(protected::link::verify_link))
        .route(
            "/translate_path/{*path}",
            get(protected::path_to_id::path_to_id),
        )
        .route(
            "/translate_id/{*id}",
            get(protected::path_to_id::id_to_path),
        )
        .with_state(state.clone())
        .nest("/speed_test", speed_test())
        .nest("/m", protected_routes(state.clone()))
        .layer(CorsLayer::very_permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(upload_timeout)))
        .layer(DefaultBodyLimit::max(upload_limit));

    let listener = tokio::net::TcpListener::bind(&addr).await.expect(&addr);
    tracing::info!("Starting server on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to serve app");
}

fn setup_tracing() {
    let args = env::args().collect::<Vec<String>>();
    let log_level_str = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let log_level = match log_level_str.to_lowercase().as_str() {
        "off" => LevelFilter::OFF,
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    let log_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("out.log")
        .expect("Failed to create log file");

    let sub = Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(log_file)
                .with_filter(log_level),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_filter(log_level),
        );

    tracing::subscriber::set_global_default(sub).expect("Failed setting default subscriber");

    if cfg!(debug_assertions) && log_level_str.is_empty() {
        tracing::info!(
            "You seem to be running in debug mode and specified no log level. You can specify a log level with `cargo r -- trace` for example."
        );
    }

    tracing::info!("Init tracing with {log_level:?} as the log level");
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
        tracing::debug!("Creating /data: {root:?}");
        fs.create_dir_all("").await?;
    }

    let public_uploads = PathBuf::from("")
        .join(".public_uploads")
        .to_string_lossy()
        .to_string();
    if !fs.exists(&public_uploads).await? {
        fs.create_dir_all(&public_uploads).await?;
    }

    Ok(())
}
