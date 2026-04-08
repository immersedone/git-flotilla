use sqlx::SqlitePool;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn pool() -> &'static SqlitePool {
    DB_POOL.get().expect("DB pool not initialised — call db::init() first")
}

pub async fn init(app: &AppHandle) -> Result<(), sqlx::Error> {
    let data_dir = app.path().app_data_dir()
        .expect("failed to resolve app data dir");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let db_path = data_dir.join("flotilla.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    tracing::info!("Opening DB at {}", db_path.display());

    let pool = SqlitePool::connect(&db_url).await?;

    // Run embedded migrations
    sqlx::migrate!("./src/db/migrations").run(&pool).await?;

    DB_POOL.set(pool).map_err(|_| {
        sqlx::Error::PoolTimedOut
    })?;

    tracing::info!("DB ready — migrations applied");
    Ok(())
}
