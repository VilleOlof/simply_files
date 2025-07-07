mod local;
mod ssh;

use async_trait::async_trait;
use axum::extract::multipart::Field;
use serde::Serialize;
use std::{fmt::Debug, io::Result, path::PathBuf};

pub use local::Local;
pub use ssh::SSH;

pub type FSStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = std::result::Result<Vec<u8>, std::io::Error>> + Send>,
>;

#[allow(unused)]
#[async_trait]
pub trait FileSystem: Send + Sync + Debug {
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn read_stream(&self, path: &str) -> Result<FSStream>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<()>;
    async fn write_stream<'a>(&self, path: &str, stream: Field<'a>) -> Result<()>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;

    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>>;
    async fn create_dir_all(&self, path: &str) -> Result<()>;
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
    async fn delete_empty_dir(&self, path: &str) -> Result<()>;

    async fn root_directory(&self) -> PathBuf;
}

#[derive(Debug, Clone, Serialize)]
pub struct FileMetadata {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
}
