use serde::Serialize;
use serde_repr::Serialize_repr;
use sqlx::{
    Result, SqlitePool,
    prelude::{FromRow, Type},
    query, query_as,
};
use time::OffsetDateTime;

#[derive(Debug, FromRow, Clone, Serialize)]
pub struct File {
    pub id: String,
    pub path: String,
    pub size: i64,
    pub download_count: i64,
    pub last_downloaded_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    access: i64,
}

#[derive(Debug, Type, Clone, Serialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum FileAccess {
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

impl File {
    #[tracing::instrument(skip(db))]
    pub async fn init(db: &SqlitePool) -> Result<()> {
        query(
            r#"
                CREATE TABLE IF NOT EXISTS files (
                    id TEXT PRIMARY KEY,
                    path TEXT NOT NULL,
                    size INTEGER DEFAULT 0,
                    download_count INTEGER DEFAULT 0,
                    last_downloaded_at DATETIME,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    access INTEGER DEFAULT 0
                );
            "#,
        )
        .execute(db)
        .await?;

        query(r#"CREATE INDEX IF NOT EXISTS idx_files_path ON files (path);"#)
            .execute(db)
            .await?;

        Ok(())
    }

    pub fn get_access(&self) -> FileAccess {
        self.access.into()
    }

    pub async fn get_via_id(db: &SqlitePool, id: &str) -> Result<Self> {
        Ok(query_as(r#"SELECT * FROM files WHERE id = ?"#)
            .bind(id)
            .fetch_one(db)
            .await?)
    }

    #[tracing::instrument(skip(db))]
    pub async fn new(db: &SqlitePool, id: &str, path: &str) -> Result<Self> {
        let file: Self = query_as(r#"INSERT INTO files (id, path) VALUES (?, ?) RETURNING *;"#)
            .bind(id)
            .bind(path)
            .fetch_one(db)
            .await?;

        Ok(file)
    }

    #[tracing::instrument(skip(db))]
    pub async fn delete(db: &SqlitePool, id: &str) -> Result<()> {
        query(r#"DELETE FROM files WHERE id = ?;"#)
            .bind(id)
            .execute(db)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, db))]
    pub async fn successful_upload(&mut self, db: &SqlitePool, size: i64) -> Result<()> {
        query(r#"UPDATE files SET size = ? WHERE id = ? "#)
            .bind(size)
            .bind(&self.id)
            .execute(db)
            .await?;

        // no need to retrive it again from the db (probably)
        // sure the data could be de-synced but here were waiting for it to be uploaded so no one will touchy touch
        self.size = size;

        Ok(())
    }

    #[tracing::instrument(skip(self, db))]
    pub async fn change_access(&mut self, db: &SqlitePool, access: FileAccess) -> Result<()> {
        query(r#"UPDATE files SET access = ? WHERE id = ?;"#)
            .bind(access.clone() as i64)
            .bind(&self.id)
            .execute(db)
            .await?;

        self.access = access.into();

        Ok(())
    }

    /// Also updates the latest download_time
    #[tracing::instrument(skip(self, db))]
    pub async fn increment_download_count(
        &self,
        db: &SqlitePool,
        amount: Option<i64>,
    ) -> Result<()> {
        query(
            r#"UPDATE files SET download_count = download_count + ?, last_downloaded_at = CURRENT_TIMESTAMP WHERE id = ?"#,
        ).bind(amount.unwrap_or(1)).bind(&self.id).execute(db).await?;

        Ok(())
    }
}
