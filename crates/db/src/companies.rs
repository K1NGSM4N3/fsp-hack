use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CompanyOfOwner {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub size: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_telegram: Option<String>,
    pub updated_at: DateTime<Utc>,
}

pub struct CompanyUpdate<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub industry: Option<&'a str>,
    pub website: Option<&'a str>,
    pub city: Option<&'a str>,
    pub size: Option<&'a str>,
    pub contact_email: Option<&'a str>,
    pub contact_phone: Option<&'a str>,
    pub contact_telegram: Option<&'a str>,
}

pub async fn insert_company(
    db: impl PgExecutor<'_>,
    id: Uuid,
    owner_user_id: Uuid,
    name: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO companies (id, owner_user_id, name) VALUES ($1, $2, $3)",
        id,
        owner_user_id,
        name,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_of_owner(
    db: impl PgExecutor<'_>,
    owner_user_id: Uuid,
) -> Result<Option<CompanyOfOwner>, sqlx::Error> {
    sqlx::query_as!(
        CompanyOfOwner,
        "SELECT id, name, description, industry, website, city, size, contact_email, contact_phone,
                contact_telegram, updated_at
         FROM companies
         WHERE owner_user_id = $1",
        owner_user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn update_company(
    db: impl PgExecutor<'_>,
    owner_user_id: Uuid,
    c: &CompanyUpdate<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE companies
         SET name = $1, description = $2, industry = $3, website = $4, city = $5, size = $6,
             contact_email = $7, contact_phone = $8, contact_telegram = $9, updated_at = now()
         WHERE owner_user_id = $10",
        c.name,
        c.description,
        c.industry,
        c.website,
        c.city,
        c.size,
        c.contact_email,
        c.contact_phone,
        c.contact_telegram,
        owner_user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
