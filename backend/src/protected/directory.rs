use std::{path::PathBuf, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Result,
};
use serde::Serialize;
use sf_core::File;

use crate::{AppState, error::SimplyError, file_system::FileMetadata};

pub async fn get_files(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    get(state, Some(&path)).await
}

pub async fn get_root(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    get(state, None).await
}

#[derive(Debug, Serialize, Clone)]
pub struct ClientFile {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub id: Option<String>,
    pub access: Option<i64>,
}

impl ClientFile {
    // this whole thing is a meh thing
    pub fn from(
        base_path: PathBuf,
        real_files: Vec<FileMetadata>,
        db_files: Vec<File>,
    ) -> Vec<ClientFile> {
        let mut files = vec![];

        for real in real_files {
            let real_path = (base_path.join(&std::path::Path::new(&real.path)))
                .to_string_lossy()
                .to_string();
            let db = match db_files
                .iter()
                .find(|f| f.path.replace("\\", "/") == real_path.replace("\\", "/"))
            {
                Some(d) => d,
                None => {
                    files.push(ClientFile {
                        path: real.path,
                        is_dir: real.is_dir,
                        size: real.size,
                        modified: real.modified,
                        id: None,
                        access: None,
                    });
                    continue;
                }
            };
            // file is still uploading (or something is wrong)
            if db.size == 0 {
                continue;
            }

            files.push(ClientFile {
                path: real.path.clone(),
                is_dir: real.is_dir,
                size: real.size,
                modified: real.modified,
                id: Some(db.id.clone()),
                access: Some(db.get_access() as i64),
            });
        }

        files
    }
}

async fn get(
    state: Arc<AppState>,
    path: Option<&str>,
) -> Result<Json<Vec<ClientFile>>, SimplyError> {
    let files = state.fs.list_dir(path.unwrap_or("")).await?;

    let db_files = crate::db::file::get_files_in_directory(&state.db, &path.unwrap_or("")).await?;

    let files = ClientFile::from(PathBuf::from(path.unwrap_or("")), files, db_files);

    let files = files
        .iter()
        // hide .public_uploads directory
        .filter(|f| !f.path.starts_with(".public_uploads"))
        .map(|f| f.clone())
        .collect();

    Ok(Json(files))
}

pub async fn add_directory(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    state.fs.create_dir_all(&path).await?;
    Ok(StatusCode::OK)
}

pub async fn delete_directory(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, SimplyError> {
    state.fs.delete_empty_dir(&path).await?;
    Ok(StatusCode::OK)
}
