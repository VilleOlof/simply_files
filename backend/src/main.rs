use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::config::Config;

mod config;
mod file_system;

#[tokio::main]
async fn main() {
    setup_tracing();

    let config = Config::read_config();
    let fs = config.get_file_system();

    fs.write("test.file", b"bom").await.unwrap();
    let data = fs.read("test.file").await.unwrap();

    assert_eq!(data, b"bom");
}

fn setup_tracing() {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(sub).expect("Failed setting default subscriber");
}
