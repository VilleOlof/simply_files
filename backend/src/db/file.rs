use serde::Serialize;
use serde_repr::Serialize_repr;
use sqlx::{
    Result, SqlitePool,
    prelude::{FromRow, Type},
    query, query_as, query_scalar,
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
                    path TEXT NOT NULL UNIQUE,
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
        Ok(query_as(r#"SELECT * FROM files WHERE id = ?;"#)
            .bind(id)
            .fetch_one(db)
            .await?)
    }

    pub async fn get_via_path(db: &SqlitePool, path: &str) -> Result<Self> {
        tracing::debug!("path: {path:?}");
        Ok(query_as(r#"SELECT * FROM files WHERE path = ?;"#)
            .bind(path)
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
        query(r#"UPDATE files SET size = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? "#)
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
        query(r#"UPDATE files SET access = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?;"#)
            .bind(access.clone() as i64)
            .bind(&self.id)
            .execute(db)
            .await?;

        self.access = access.into();

        Ok(())
    }

    #[tracing::instrument(skip(self, db))]
    pub async fn rename(&mut self, db: &SqlitePool, new_path: &str) -> Result<()> {
        query(r#"UPDATE files SET path = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?;"#)
            .bind(new_path)
            .bind(&self.id)
            .execute(db)
            .await?;

        self.path = new_path.to_string();

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

    #[tracing::instrument(skip(db))]
    pub async fn get_bytes_stored(db: &SqlitePool) -> Result<u64> {
        let bytes: u64 = query_scalar(r#"SELECT SUM(size) FROM files"#)
            .fetch_one(db)
            .await?;
        Ok(bytes)
    }

    #[tracing::instrument(skip(db))]
    pub async fn get_files_in_directory(db: &SqlitePool, path: &str) -> Result<Vec<Self>> {
        let files = if path.is_empty() {
            query_as(r#"SELECT * FROM files WHERE instr(path, '/') = 0"#)
                .fetch_all(db)
                .await?
        } else {
            query_as(
                r#"
                    SELECT * FROM files
                        WHERE path LIKE ?1
                        AND instr(substr(path, ?2 + 2), '/') = 0;
                "#,
            )
            .bind(&format!("{}/%", &path))
            .bind(path.len() as i64)
            .fetch_all(db)
            .await?
        };

        Ok(files)
    }

    #[tracing::instrument(skip(db))]
    pub async fn get_all_files(db: &SqlitePool) -> Result<Vec<Self>> {
        Ok(query_as(r#"SELECT * FROM files"#).fetch_all(db).await?)
    }

    #[tracing::instrument(skip(db))]
    pub async fn get_total_amount_of_files(db: &SqlitePool) -> Result<i64> {
        Ok(query_scalar(r#"SELECT COUNT(*) FROM files"#)
            .fetch_one(db)
            .await?)
    }
}
