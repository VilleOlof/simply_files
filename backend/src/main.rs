use once_cell::sync::OnceCell;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{config::Config, file_system::FileSystem};

mod config;
mod file_system;

static CONFIG: OnceCell<Config> = OnceCell::new();
static FS: OnceCell<Box<dyn FileSystem>> = OnceCell::new();

pub fn config() -> &'static Config {
    CONFIG.get().expect("CONFIG not init")
}
pub fn fs() -> &'static Box<dyn FileSystem> {
    FS.get().expect("CONFIG not init")
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let init_config = Config::read_config();
    CONFIG.set(init_config).expect("Failed to set CONFIG");
    let file_system = config().get_file_system();
    FS.set(file_system).expect("Failed to set FS");

    fs().write("test.file", b"bom").await.unwrap();
    let data = fs().read("test.file").await.unwrap();

    assert_eq!(data, b"bom");

    fs().exists("test.file").await.unwrap();
}

fn setup_tracing() {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(sub).expect("Failed setting default subscriber");
}
