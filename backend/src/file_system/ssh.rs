use async_trait::async_trait;
use ssh2::Session;
use std::{
    io::{Read, Result, Write},
    net::TcpStream,
    path::Path,
    sync::Mutex,
    time::Duration,
};

use crate::file_system::{FileMetadata, FileSystem};

pub struct SSH {
    #[allow(unused)]
    session: Mutex<Session>,
    sftp: ssh2::Sftp,
    root: String,
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
    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Reading {full_path:?}");
        let mut file = self.sftp.open(Path::new(&full_path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Writing to {full_path:?}");
        let mut file = self.sftp.create(Path::new(&full_path))?;
        file.write_all(data)?;
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Deleting {full_path:?}");
        self.sftp.unlink(Path::new(&full_path))?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Checking if {full_path:?} exists");
        Ok(self.sftp.stat(Path::new(&full_path)).is_ok())
    }

    async fn list_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Listing {full_path:?}");
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

    async fn create_dir_all(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path);
        tracing::debug!("[SSH]: Creating all dirs for {full_path:?}");
        let parts = Path::new(&full_path).ancestors().collect::<Vec<_>>();
        for ancestor in parts.iter().rev() {
            let _ = self.sftp.mkdir(ancestor, 0o755); // ignore already exists
        }
        Ok(())
    }

    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let full_from = self.full_path(from);
        let full_to = self.full_path(to);
        tracing::debug!("[SSH]: Renaming {full_from:?} to {full_to:?}");
        self.sftp
            .rename(Path::new(&full_from), Path::new(&full_to), None)?;
        Ok(())
    }
}
