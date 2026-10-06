//! PostgreSQL access: one module per domain area. Every query is a `sqlx` macro checked against
//! the schema at compile time; offline metadata lives in `.sqlx/` (`make db-prepare`).

pub mod assessment;
pub mod candidates;
pub mod companies;
pub mod consents;
pub mod contacts;
pub mod fsp;
pub mod grades;
pub mod guardians;
pub mod integrations;
pub mod invitations;
pub mod market;
pub mod me;
pub mod moderation;
pub mod needs;
pub mod notifications;
pub mod pool;
pub mod proctoring;
pub mod reports;
pub mod seed;
pub mod survey;
pub mod tasks;
pub mod trust;
pub mod types;
pub mod users;
pub mod vacancies;

use sqlx::{Connection, PgPool, migrate::Migrator};

pub static MIGRATOR: Migrator = sqlx::migrate!("../../db/migrations");

pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.acquire().await?.ping().await
}

/// SQLSTATE 23505: the caller maps it to a domain conflict (an active invitation, a repeated application).
pub fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|d| d.is_unique_violation())
}
