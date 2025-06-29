use async_trait::async_trait;
use ssh2::Session;
use std::{
    fmt::Debug,
    io::{Read, Result, Write},
    net::TcpStream,
    path::Path,
    pin::Pin,
    sync::Mutex,
    time::Duration,
};
use tokio_stream::{Stream, StreamExt};

use crate::file_system::{FileMetadata, FileSystem};

pub struct SSH {
    #[allow(unused)]
    session: Mutex<Session>,
    sftp: ssh2::Sftp,
    root: String,
}

impl Debug for SSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SSH")
    }
}

impl SSH {
    pub fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        root: impl Into<String>,
    ) -> Result<Self> {
        tracing::debug!("Connecting to remote SSH");
        let tcp = TcpStream::connect((host, port))?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

        let mut session = Session::new().unwrap();
        session.set_tcp_stream(tcp);
        tracing::debug!("Started SSH handshake");
        session.handshake()?;
        tracing::debug!("Authenticating SSH");
        session.userauth_password(username, password)?;

        if !session.authenticated() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "SSH auth failed",
            ));
        }

        tracing::debug!("Connecting via SFTP");
        let sftp = session.sftp()?;
        Ok(Self {
            session: Mutex::new(session),
            sftp,
            root: root.into(),
        })
    }

    fn full_path(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.root.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[async_trait]
impl FileSystem for SSH {
    #[tracing::instrument]
    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let mut file = self.sftp.open(Path::new(&full_path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[tracing::instrument(skip(data))]
    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let mut file = self.sftp.create(Path::new(&full_path))?;
        file.write_all(data)?;
        Ok(())
    }

    #[tracing::instrument(skip(stream))]
    async fn write_stream(
        &self,
        path: &str,
        mut stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + Send>>,
    ) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("Streaming to {:?}", full_path);
        let mut file = self.sftp.create(Path::new(&full_path))?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        Ok(())
    }

    #[tracing::instrument]
    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        self.sftp.unlink(Path::new(&full_path))?;
        Ok(())
    }

    #[tracing::instrument]
    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        Ok(self.sftp.stat(Path::new(&full_path)).is_ok())
    }

    #[tracing::instrument]
    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let entries = self.sftp.readdir(Path::new(&full_path))?;
        let mut result = Vec::new();

        for (pathbuf, stat) in entries {
            result.push(FileMetadata {
                path: pathbuf
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
                modified: stat.mtime.unwrap_or(0),
            });
        }

        Ok(result)
    }

    #[tracing::instrument]
    async fn create_dir_all(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let parts = Path::new(&full_path).ancestors().collect::<Vec<_>>();
        for ancestor in parts.iter().rev() {
            let _ = self.sftp.mkdir(ancestor, 0o755); // ignore already exists
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);
        tracing::debug!("{:?} to {:?}", from_path, to_path);
        self.sftp
            .rename(Path::new(&from_path), Path::new(&to_path), None)?;
        Ok(())
    }
}
