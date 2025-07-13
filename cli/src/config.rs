use std::{
    fs::{create_dir_all, exists, read_to_string, write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub hosts: Vec<Host>,
    pub default_host: Option<String>,
    pub link_host: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Host {
    pub name: String,
    pub url: String,
    pub token: Option<String>,
}

impl Config {
    pub fn get() -> Config {
        let path = Self::path();
        if !exists(&path).expect("Failed to check if config exists") {
            let config = Config::default();
            config.save();
            return config;
        }

        let contents =
            read_to_string(&path).expect(&format!("Failed to read config file: {path:?}"));
        toml::from_str(&contents).expect("Failed to deserialize into toml")
    }

    pub fn save(&self) {
        tracing::debug!("Saving config");

        let path = Self::path();
        if !exists(&path.parent().expect("No parent in config path"))
            .expect("Failed to check if config dir exists")
        {
            create_dir_all(&path).expect("Failed to create config dir");
        }

        let contents = toml::to_string_pretty(self).expect("Failed to serialize into toml");
        write(&path, &contents).expect(&format!("Failed to write config file: {path:?}"))
    }

    pub fn path() -> PathBuf {
        ProjectDirs::from("com", "simply_files", "sf_cli")
            .expect("Failed to get project/config directory")
            .config_dir()
            .to_path_buf()
            .join("config.toml")
    }
}
