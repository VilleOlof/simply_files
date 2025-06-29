use async_trait::async_trait;
use std::{
    fmt::Debug,
    io::Result,
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};
use tokio::{fs, task};

use crate::file_system::{FileMetadata, FileSystem};

pub struct Local {
    pub root: PathBuf,
}

impl Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Local")
    }
}

impl Local {
    fn full_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
}

#[async_trait]
impl FileSystem for Local {
    #[tracing::instrument]
    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        fs::read(full_path).await
    }

    #[tracing::instrument(skip(data))]
    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(full_path, data).await
    }

    #[tracing::instrument]
    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        fs::remove_file(full_path).await
    }

    #[tracing::instrument]
    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        Ok(full_path.exists())
    }

    #[tracing::instrument]
    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let full_path = self.root.join(path);
        tracing::debug!("{:?}", full_path);
        let entries = task::spawn_blocking(move || -> Result<Vec<FileMetadata>> {
            let mut result = Vec::new();
            for entry in std::fs::read_dir(full_path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                result.push(FileMetadata {
                    path: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata
                        .modified()?
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::new(0, 0))
                        .as_secs(),
                });
            }
            Ok(result)
        })
        .await
        .unwrap(); // unwrap task result (safe here unless panicked)

        entries
    }

    #[tracing::instrument]
    async fn create_dir_all(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        fs::create_dir_all(full_path).await
    }

    #[tracing::instrument]
    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);
        tracing::debug!("{:?} to {:?}", from_path, to_path);
        fs::rename(from_path, to_path).await
    }
}
