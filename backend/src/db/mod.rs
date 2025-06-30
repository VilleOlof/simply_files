use sqlx::{Result, SqlitePool};

pub mod file;

pub async fn init(db: &SqlitePool) -> Result<()> {
    file::File::init(db).await?;
    Ok(())
}
