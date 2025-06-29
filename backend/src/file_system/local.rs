use async_trait::async_trait;
use std::{
    io::Result,
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};
use tokio::{fs, task};

use crate::file_system::{FileMetadata, FileSystem};

pub struct Local {
    pub root: PathBuf,
}

#[async_trait]
impl FileSystem for Local {
    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.root.join(path);
        tracing::debug!("[Local]: Reading {full_path:?}");
        fs::read(full_path).await
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.root.join(path);
        tracing::debug!("[Local]: Writing to {full_path:?}");
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(full_path, data).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.root.join(path);
        tracing::debug!("[Local]: Deleting {full_path:?}");
        fs::remove_file(full_path).await
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.root.join(path);
        tracing::debug!("[Local]: Checking if {full_path:?} exists");
        Ok(full_path.exists())
    }

    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let base = self.root.join(path);
        tracing::debug!("[Local]: Listing {base:?}");
        let entries = task::spawn_blocking(move || -> Result<Vec<FileMetadata>> {
            let mut result = Vec::new();
            for entry in std::fs::read_dir(base)? {
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

    async fn create_dir_all(&self, path: &str) -> Result<()> {
        let full_path = self.root.join(path);
        tracing::debug!("[Local]: Creating all dirs for {full_path:?}");
        fs::create_dir_all(full_path).await
    }

    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.root.join(from);
        let to_path = self.root.join(to);
        tracing::debug!("[Local]: Renaming {from_path:?} to {to_path:?}");
        fs::rename(from_path, to_path).await
    }
}
