use std::time::Duration;

use sqlx::{
    Connection, PgPool,
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

/// Default connections per replica (`DATABASE_POOL_SIZE`): about as many queries as the
/// database host's cores run at once. A larger pool only moves the queue into Postgres, where it
/// stretches tail latency instead of throughput (README.md section 12.4).
pub const POOL_SIZE: u32 = 16;
/// A connection idle at least this long is pinged before use.
const PING_AFTER_IDLE: Duration = Duration::from_secs(1);

/// sqlx pings every connection it hands out by default: a second round trip per query. Only a
/// connection that sat idle is pinged here; one in steady use proved alive moments ago, and if
/// the server went away since, the query fails and the pool drops the connection.
pub fn pool(database_url: &str, size: u32) -> Result<PgPool, DbSetupError> {
    let options: PgConnectOptions = database_url.parse()?;
    Ok(PgPoolOptions::new()
        .max_connections(size)
        .test_before_acquire(false)
        .before_acquire(|conn, meta| {
            Box::pin(async move {
                if meta.idle_for >= PING_AFTER_IDLE {
                    conn.ping().await?;
                }
                Ok(true)
            })
        })
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
