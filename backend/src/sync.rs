use std::{io, path::PathBuf, pin::Pin, sync::Arc};

use crate::{AppState, db::file::File, generate_id};

pub async fn sync_files(state: Arc<AppState>) -> Result<(), SyncError> {
    sync_from_db(&state).await?;
    sync_from_files(&state).await?;
    Ok(())
}

async fn sync_from_db(state: &AppState) -> Result<(), SyncError> {
    let files = File::get_all_files(&state.db).await?;

    let mut count = 0;
    for file in files {
        // if any of the db files doesnt exist on the actual system
        // remove it from the database
        if !state.fs.exists(&file.path).await? {
            File::delete(&state.db, &file.id).await?;
            tracing::info!(
                "Deleted '{:?}' from database to sync with file system",
                PathBuf::from(file.path).file_name()
            );
            count += 1;
        }
    }

    if count > 0 {
        tracing::info!("Deleted {count} files from the database");
    }
    Ok(())
}

async fn sync_from_files(state: &Arc<AppState>) -> Result<(), SyncError> {
    // iterate over all directories from state.fs.root_directory()
    // and transform the path for database query (full_current - root) (/ NOT \)
    // then insert the files with a new ID, access = 0 etc.
    let root_path = state
        .fs
        .root_directory()
        .await
        .to_string_lossy()
        .to_string();

    let root_files = state.fs.list_dir("").await?;

    let cb: Arc<AsyncFn> =
        Arc::new(|state, path, root, size| Box::pin(handle_entry(state.clone(), path, root, size)));

    let old_file_count = File::get_total_amount_of_files(&state.db).await?;

    for file in root_files {
        let full_path = PathBuf::from(&root_path).join(&file.path);
        if file.is_dir {
            visit_dirs(full_path, state.clone(), root_path.to_string(), cb.clone()).await?;
            continue;
        }

        handle_entry(
            state.clone(),
            full_path.to_string_lossy().to_string(),
            root_path.to_string(),
            file.size as i64,
        )
        .await?;
    }

    let new_file_count = File::get_total_amount_of_files(&state.db).await?;
    if new_file_count > old_file_count {
        tracing::info!(
            "Added {:?} new file entries into the database",
            new_file_count - old_file_count
        );
    }

    Ok(())
}

async fn handle_entry(
    state: Arc<AppState>,
    path: String,
    root: String,
    size: i64,
) -> Result<(), SyncError> {
    let path = path.replace(r#"\\"#, "/").replace(r#"\"#, "/");
    let path = PathBuf::from(&path);

    let db_path = match path.strip_prefix(&root) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            tracing::debug!("failed to strip root: {path:?}");
            return Ok(());
        }
    };
    match File::get_via_path(&state.db, &db_path).await {
        Ok(_) => return Ok(()), // exists so we can skip doing anything
        Err(_) => {
            // doesnt exist, so we add. we can give it a -1 total chunks since its from syncing
            let mut file = File::new(&state.db, &generate_id(None), &db_path, -1).await?;
            file.successful_upload(&state.db, size).await?;

            tracing::info!(
                "Added '{:?}' in database to sync with file system",
                &db_path
            );
        }
    };

    Ok(())
}

type AsyncFn = dyn Fn(
        &Arc<AppState>,
        String,
        String,
        i64,
    ) -> Pin<Box<dyn Future<Output = Result<(), SyncError>> + Send + '_>>
    + Send
    + Sync;

fn visit_dirs(
    dir: PathBuf,
    state: Arc<AppState>,
    root: String,
    cb: Arc<AsyncFn>,
) -> Pin<Box<dyn Future<Output = Result<(), SyncError>> + Send>> {
    Box::pin(visit_dirs_inner(dir, state, root, cb))
}

async fn visit_dirs_inner(
    dir: PathBuf,
    state: Arc<AppState>,
    root: String,
    cb: Arc<AsyncFn>,
) -> Result<(), SyncError> {
    if dir.is_dir() {
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(path, state.clone(), root.clone(), cb.clone()).await?;
            } else {
                cb(
                    &state,
                    path.to_string_lossy().to_string(),
                    root.clone(),
                    entry.metadata().await?.len() as i64,
                )
                .await?;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SyncError {
    IO(io::Error),
    DB(sqlx::Error),
}

impl From<io::Error> for SyncError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<sqlx::Error> for SyncError {
    fn from(value: sqlx::Error) -> Self {
        Self::DB(value)
    }
}
