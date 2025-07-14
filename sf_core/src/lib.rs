use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use sqlx::{FromRow, Type};
use time::OffsetDateTime;

// TODO: Probably need to move some other structs like FileLinks & common stuff between the CLI & backend

pub mod simply_packet;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct File {
    pub id: String,
    pub path: String,
    pub size: i64,
    pub download_count: i64,
    pub last_downloaded_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    access: i64,
    pub chunk_index: i64,
    pub total_chunks: i64,
}

#[derive(Debug, Type, Clone, Serialize_repr, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FileAccess {
    #[default]
    Private = 0,
    Public = 1,
}

impl From<i64> for FileAccess {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Private,
            1 => Self::Public,
            _ => Self::Private,
        }
    }
}

impl From<FileAccess> for i64 {
    fn from(value: FileAccess) -> Self {
        match value {
            FileAccess::Private => 0,
            FileAccess::Public => 1,
        }
    }
}

impl FromStr for FileAccess {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(FileAccess::Private);
        }

        let num = match s.parse::<i64>() {
            Ok(n) => n,
            Err(err) => match s.to_lowercase().as_str() {
                "private" => 0,
                "public" => 1,
                _ => return Err(format!("Failed to convert input to FileAccess: {err:?}")),
            },
        };
        Ok(num.into())
    }
}

impl ToString for FileAccess {
    fn to_string(&self) -> String {
        String::from(match self {
            FileAccess::Private => "Private",
            FileAccess::Public => "Public",
        })
    }
}

impl File {
    pub fn get_access(&self) -> FileAccess {
        self.access.into()
    }

    pub fn set_access(&mut self, access: FileAccess) {
        self.access = access as i64;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewData {
    pub size: i64,
    pub file_name: String,
    pub id: String,
    pub created_at: OffsetDateTime,
    pub mime_type: String,
    pub access: i64,
    pub path: Option<String>,
    pub cant_preview: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
}
