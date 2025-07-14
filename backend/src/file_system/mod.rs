mod local;
mod ssh;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sf_core::FileMetadata;
use std::{
    fmt::Debug,
    io::{Result, Seek, Write},
    path::PathBuf,
};

pub use local::Local;
pub use ssh::SSH;

pub type FSStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = std::result::Result<Vec<u8>, std::io::Error>> + Send>,
>;

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}
pub type FileHandler = Box<dyn WriteSeek + Send + Sync>;

#[allow(unused)]
#[async_trait]
pub trait FileSystem: Send + Sync + Debug {
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn read_stream(&self, path: &str) -> Result<FSStream>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<()>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;
    async fn get_file_handler(&self, path: &str) -> Result<FileHandler>;

    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>>;
    async fn create_dir_all(&self, path: &str) -> Result<()>;
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
    async fn delete_empty_dir(&self, path: &str) -> Result<()>;

    async fn root_directory(&self) -> PathBuf;
}
