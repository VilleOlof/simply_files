use serde::Deserialize;
use std::path::PathBuf;

use crate::file_system::{FileSystem, Local, SSH};

#[derive(Debug, Deserialize)]
pub struct Config {
    file_system: WhichFileSystem,
    port: u16,

    ssh: Option<SSHConfig>,
    local: Option<LocalConfig>,
}

#[derive(Debug, Deserialize)]
pub enum WhichFileSystem {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "ssh")]
    SSH,
}

#[derive(Debug, Deserialize)]
struct SSHConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    root: String,
}

#[derive(Debug, Deserialize)]
struct LocalConfig {
    root: String,
}

impl Config {
    const CONFIG_FILE: &'static str = "config.toml";

    pub fn read_config() -> Self {
        let str = std::fs::read_to_string(Config::CONFIG_FILE)
            .expect("Failed to read config file (config.toml)");

        tracing::debug!("Read config, deserializing into Config...");

        toml::from_str(&str).expect("Invalid toml in config")
    }

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
                        &sub_config.root,
                    )
                    .unwrap(),
                )
            }
        }
    }
}
