use async_trait::async_trait;
use ssh2::Session;
use std::{
    fmt::Debug,
    io::{Read, Result, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};

use crate::{
    config::{SSHPassword, SSHPublicKey},
    file_system::{FSStream, FileMetadata, FileSystem},
};

pub struct SSH {
    #[allow(unused)]
    session: Arc<Mutex<Session>>,
    sftp: ssh2::Sftp,
    root: String,
}

impl Debug for SSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SSH")
    }
}

impl SSH {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password_config: &Option<SSHPassword>,
        public_key_config: &Option<SSHPublicKey>,
        root: impl Into<String>,
    ) -> Result<Self> {
        tracing::debug!("Connecting to remote SSH");
        let tcp = TcpStream::connect((host, port))?;
        tcp.set_read_timeout(None)?;
        tcp.set_write_timeout(None)?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        tracing::debug!("Started SSH handshake");
        session.handshake()?;

        tracing::debug!("Authenticating SSH");
        if let Some(config) = public_key_config {
            session.userauth_pubkey_file(
                username,
                config.public_key.as_deref(),
                &config.private_key,
                config.pass_phrase.as_deref(),
            )?;
            tracing::debug!("Authenticated via public_key");
        } else if let Some(config) = password_config {
            session.userauth_password(username, &config.password)?;
            tracing::debug!("Authenticated via password");
        }

        if !session.authenticated() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "SSH auth failed",
            ));
        }

        session.set_keepalive(true, 30);

        tracing::debug!("Connecting via SFTP");
        let sftp = session.sftp()?;
        let session = Arc::new(Mutex::new(session));

        let ssh = Self {
            session,
            sftp,
            root: root.into(),
        };

        ssh.start_keepalive().await;

        Ok(ssh)
    }

    pub async fn start_keepalive(&self) {
        let session = self.session.clone();
        tokio::spawn(async move {
            loop {
                let remaining = {
                    let guard = session.lock().await;
                    match guard.keepalive_send() {
                        Ok(r) => r,
                        Err(err) => {
                            tracing::error!("SSH keepalive failed: {err:?}");
                            break;
                        }
                    }
                };

                let remaining = Duration::from_secs(remaining.max(1) as u64);
                tokio::time::sleep(remaining).await;
            }
        });
        tracing::debug!("Started SSH keepalive loop");
    }

    fn full_path(&self, path: &str) -> String {
        if path == "" {
            return self.root.clone();
        };

        let path = PathBuf::from(&self.root).join(path);

        path.to_string_lossy().to_string().replace("\\", "/")
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

    #[tracing::instrument]
    async fn read_stream(&self, path: &str) -> Result<FSStream> {
        let full_path = self.full_path(path);
        tracing::debug!("Streaming from {:?}", full_path);

        let file = self.sftp.open(Path::new(&full_path))?;

        let (tx, rx) =
            tokio::sync::mpsc::channel::<std::result::Result<Vec<u8>, std::io::Error>>(16);

        tokio::task::spawn_blocking(move || {
            const CHUNK_SIZE: usize = 8192;
            let mut file = file;
            let mut buffer = vec![0u8; CHUNK_SIZE];

            loop {
                match file.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(bytes_read) => {
                        let chunk = buffer[..bytes_read].to_vec();
                        if tx.blocking_send(Ok(chunk)).is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(Err(e));
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
    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let full_path = self.full_path(path);
        tracing::debug!("{:?}", full_path);
        let stat = self.sftp.stat(&Path::new(&full_path))?;
        Ok(FileMetadata {
            path: full_path,
            is_dir: stat.is_dir(),
            size: stat.raw().filesize,
            modified: stat.raw().mtime as u64,
        })
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

    #[tracing::instrument]
    async fn delete_empty_dir(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(&path);
        tracing::debug!("{:?}", full_path);
        let is_empty = self.list_dir(&path).await?.is_empty();

        if is_empty {
            self.sftp.rmdir(Path::new(&full_path))?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Tried to delete an empty directory",
            ))
        }
    }

    async fn root_directory(&self) -> PathBuf {
        PathBuf::from(&self.root)
    }
}
