use async_trait::async_trait;
use std::{
    fmt::Debug,
    io::Result,
    path::PathBuf,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{fs, task};
use tokio_stream::{Stream, StreamExt};

use crate::file_system::{FSStream, FileMetadata, FileSystem};

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
        if path == "" {
            return self.root.clone();
        };

        let path = self.root.join(path);
        let path = PathBuf::from(path.to_string_lossy().to_string().replace("\\", "/"));

        path
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

    #[tracing::instrument]
    async fn read_stream(&self, path: &str) -> Result<FSStream> {
        let full_path = self.full_path(path);
        tracing::debug!("Streaming from {:?}", full_path);

        let file = fs::File::open(&full_path).await?;

        const CHUNK_SIZE: usize = 8192;
        let mut reader = tokio::io::BufReader::new(file);

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            loop {
                let mut chunk = vec![0u8; CHUNK_SIZE];
                match reader.read(&mut chunk).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        chunk.truncate(n);
                        if tx.send(Ok(chunk)).await.is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Box::pin(stream))
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

    #[tracing::instrument(skip(stream))]
    async fn write_stream(
        &self,
        path: &str,
        mut stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + Send>>,
    ) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("Streaming to {:?}", full_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(&full_path).await?;

        use tokio::io::AsyncWriteExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }

        Ok(())
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
    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let stat = fs::metadata(&full_path).await?;
        Ok(FileMetadata {
            path: full_path.to_string_lossy().to_string(),
            is_dir: stat.is_dir(),
            size: stat.len(),
            modified: stat
                .modified()
                .unwrap_or(SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        })
    }

    #[tracing::instrument]
    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let entries = task::spawn_blocking(move || -> Result<Vec<FileMetadata>> {
            let mut result = Vec::new();
            for entry in std::fs::read_dir(&full_path)? {
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
        .expect("list dir task panic'd?");

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

    #[tracing::instrument]
    async fn delete_empty_dir(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(&path);
        tracing::debug!("{:?}", full_path);
        let is_empty = self.list_dir(&path).await?.is_empty();

        if is_empty {
            fs::remove_dir(full_path).await?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Tried to delete an empty directory",
            ))
        }
    }

    async fn root_directory(&self) -> PathBuf {
        self.root.clone()
    }
}
