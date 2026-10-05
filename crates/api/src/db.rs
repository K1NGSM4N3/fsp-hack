use sqlx::{
    PgPool,
    migrate::{Migrate, MigrateError},
    postgres::{PgConnectOptions, PgPoolOptions},
};

#[derive(Debug, thiserror::Error)]
pub enum DbSetupError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migration(#[from] MigrateError),
}

pub fn pool(database_url: &str) -> Result<PgPool, DbSetupError> {
    let options: PgConnectOptions = database_url.parse()?;
    Ok(PgPoolOptions::new()
        .max_connections(16)
        .connect_lazy_with(options))
}

/// Concurrent replicas are safe: `Migrator::run` holds a Postgres advisory lock while applying.
pub async fn migrate(pool: &PgPool) -> Result<(), DbSetupError> {
    let mut conn = pool.acquire().await?;
    // The history table does not exist before the very first run.
    let before = conn.list_applied_migrations().await.unwrap_or_default();
    db::MIGRATOR.run(&mut *conn).await?;
    for m in db::MIGRATOR
        .iter()
        .filter(|m| before.iter().all(|a| a.version != m.version))
    {
        tracing::info!(version = m.version, name = %m.description, "migration applied");
    }
    Ok(())
}
