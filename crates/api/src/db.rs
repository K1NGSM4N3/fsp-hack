use deadpool_postgres::{BuildError, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use tokio_postgres::NoTls;

mod embedded {
    refinery::embed_migrations!("../../db/migrations");
}

#[derive(Debug, thiserror::Error)]
pub enum DbSetupError {
    #[error("DATABASE_URL: {0}")]
    Url(#[from] tokio_postgres::Error),
    #[error("pool: {0}")]
    Build(#[from] BuildError),
    #[error("pool: {0}")]
    Pool(#[from] PoolError),
    #[error("migration: {0}")]
    Migration(#[from] refinery::Error),
}

pub fn pool(database_url: &str) -> Result<Pool, DbSetupError> {
    let pg: tokio_postgres::Config = database_url.parse()?;
    let manager = Manager::from_config(
        pg,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    Ok(Pool::builder(manager).max_size(16).build()?)
}

pub async fn migrate(pool: &Pool) -> Result<(), DbSetupError> {
    let mut client = pool.get().await?;
    let client: &mut tokio_postgres::Client = &mut client;
    let report = embedded::migrations::runner().run_async(client).await?;
    for m in report.applied_migrations() {
        tracing::info!(migration = %m, "applied");
    }
    Ok(())
}
