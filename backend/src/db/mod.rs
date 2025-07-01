use sqlx::{Result, SqlitePool};

pub mod file;
pub mod links;

pub async fn init(db: &SqlitePool) -> Result<()> {
    file::File::init(db).await?;
    links::FileLink::init(db).await?;
    Ok(())
}
