use serde::Serialize;
use sqlx::{Result, SqlitePool, prelude::FromRow, query, query_as};
use time::OffsetDateTime;

use crate::generate_id;

#[derive(Debug, FromRow, Clone, Serialize)]
pub struct FileLink {
    pub id: String,
    pub uploaded_file: Option<String>,
    pub uploaded_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl FileLink {
    #[tracing::instrument(skip(db))]
    pub async fn init(db: &SqlitePool) -> Result<()> {
        query(
            r#"
                CREATE TABLE IF NOT EXISTS links (
                    id TEXT PRIMARY KEY,
                    uploaded_file TEXT,
                    uploaded_at DATETIME,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
            "#,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(db))]
    pub async fn new(db: &SqlitePool) -> Result<FileLink> {
        let id = generate_id(None);

        let link = query_as(r#"INSERT INTO links (id) VALUES (?) RETURNING *;"#)
            .bind(id)
            .fetch_one(db)
            .await?;

        Ok(link)
    }

    #[tracing::instrument(skip(db))]
    pub async fn get_via_id(db: &SqlitePool, id: &str) -> Result<FileLink> {
        Ok(query_as(r#"SELECT * FROM links WHERE id = ?;"#)
            .bind(id)
            .fetch_one(db)
            .await?)
    }

    #[tracing::instrument(skip(self, db))]
    pub async fn uploaded_with(&self, db: &SqlitePool, file_id: &str) -> Result<()> {
        query(
            r#"UPDATE links SET uploaded_file = ?, uploaded_at = CURRENT_TIMESTAMP WHERE id = ?"#,
        )
        .bind(file_id)
        .bind(&self.id)
        .execute(db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(db))]
    pub async fn get_unused_links(db: &SqlitePool) -> Result<Vec<FileLink>> {
        Ok(
            query_as(r#"SELECT * FROM links WHERE uploaded_file IS NULL;"#)
                .fetch_all(db)
                .await?,
        )
    }

    #[tracing::instrument(skip(self))]
    pub fn is_valid_to_use(&self) -> bool {
        if self.uploaded_file.is_some() && self.uploaded_at.is_some() {
            return false;
        }

        true
    }

    #[tracing::instrument(skip(db))]
    pub async fn delete(db: &SqlitePool, id: &str) -> Result<()> {
        query(r#"DELETE FROM links WHERE id = ?;"#)
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }
}
