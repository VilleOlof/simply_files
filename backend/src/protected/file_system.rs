use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;

use crate::{AppState, config::WhichFileSystem};

#[derive(Debug, Serialize)]
pub struct FileSystemInfo {
    which: String,
    about: String,
}

pub async fn get_file_system(State(state): State<Arc<AppState>>) -> Json<FileSystemInfo> {
    let info = match state.config.file_system {
        WhichFileSystem::Local => {
            let config = state.config.local.as_ref().expect("Invalid config");
            FileSystemInfo {
                which: "Local".into(),
                about: format!("{}", config.root),
            }
        }
        WhichFileSystem::SSH => {
            let config = state.config.ssh.as_ref().expect("Invalid config");
            FileSystemInfo {
                which: "SSH".into(),
                about: format!(
                    "{}:*****@{}:{} | {}",
                    config.username, config.host, config.port, config.root
                ),
            }
        }
    };

    Json(info)
}
