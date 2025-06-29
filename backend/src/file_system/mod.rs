mod local;
mod ssh;

use async_trait::async_trait;
use std::{fmt::Debug, io::Result};

pub use local::Local;
pub use ssh::SSH;

#[allow(unused)]
#[async_trait]
pub trait FileSystem: Send + Sync + Debug {
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<()>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn exists(&self, path: &str) -> Result<bool>;

    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>>;
    async fn create_dir_all(&self, path: &str) -> Result<()>;
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
}
