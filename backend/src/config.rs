use serde::Deserialize;
use std::path::PathBuf;

use crate::file_system::{FileSystem, Local, SSH};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub file_system: WhichFileSystem,
    pub addr: String,
    pub backend_url: Option<String>,
    pub web_url: Option<String>,
    pub db: String,
    pub token: String,
    pub upload_limit: usize,
    pub storage_limit: usize,
    pub upload_timeout: u64,

    pub ssh: Option<SSHConfig>,
    pub local: Option<LocalConfig>,
}

#[derive(Debug, Deserialize)]
pub enum WhichFileSystem {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "ssh")]
    SSH,
}

#[derive(Debug, Deserialize)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub root: String,

    pub password: Option<SSHPassword>,
    pub public_key: Option<SSHPublicKey>,
}

#[derive(Debug, Deserialize)]
pub struct SSHPassword {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SSHPublicKey {
    pub public_key: Option<PathBuf>,
    pub private_key: PathBuf,
    pub pass_phrase: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocalConfig {
    pub root: String,
}

impl WhichFileSystem {
    #[allow(unused)]
    pub fn to_string(&self) -> String {
        (match self {
            &Self::Local => "Local",
            &Self::SSH => "SSH",
        })
        .to_string()
    }
}

impl Config {
    const CONFIG_FILE: &'static str = "config.toml";

    pub fn read_config() -> Self {
        let str = std::fs::read_to_string(Config::CONFIG_FILE)
            .expect("Failed to read config file (config.toml)");

        tracing::debug!("Read config, deserializing into Config...");

        toml::from_str(&str).expect("Invalid toml in config")
    }

    #[tracing::instrument(skip(self))]
    pub fn get_file_system(&self) -> Box<dyn FileSystem> {
        match self.file_system {
            WhichFileSystem::Local => {
                let sub_config = self.local.as_ref().expect("No local config");
                tracing::info!("Creating a 'Local' file system");
                Box::new(Local {
                    root: PathBuf::from(&sub_config.root),
                })
            }
            WhichFileSystem::SSH => {
                let sub_config = self.ssh.as_ref().expect("No ssh config");
                tracing::info!("Creating a 'SSH' file system (SFTP)");
                Box::new(
                    SSH::connect(
                        &sub_config.host,
                        sub_config.port,
                        &sub_config.username,
                        &sub_config.password,
                        &sub_config.public_key,
                        &sub_config.root,
                    )
                    .expect("Failed to connect to SSH host"),
                )
            }
        }
    }
}
