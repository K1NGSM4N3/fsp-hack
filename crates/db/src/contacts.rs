use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CandidateSecrets {
    pub full_name_enc: Option<Vec<u8>>,
    pub contacts_enc: Option<Vec<u8>>,
}

pub struct ContactAccess<'a> {
    pub id: Uuid,
    pub employer_user_id: Uuid,
    pub company_id: Uuid,
    pub candidate_id: Uuid,
    pub basis: &'a str,
    pub basis_id: Uuid,
    pub ip: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

pub async fn accepted_invitation_basis(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM invitations
         WHERE company_id = $1 AND candidate_id = $2 AND status = 'accepted'
         ORDER BY responded_at DESC
         LIMIT 1",
        company_id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn application_basis(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT a.id FROM applications a
         JOIN vacancies v ON v.id = a.vacancy_id
         WHERE v.company_id = $1 AND a.candidate_id = $2 AND a.status <> 'withdrawn'
         ORDER BY a.created_at DESC
         LIMIT 1",
        company_id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn candidate_secrets(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<CandidateSecrets, sqlx::Error> {
    sqlx::query_as!(
        CandidateSecrets,
        "SELECT full_name_enc, contacts_enc FROM candidates WHERE user_id = $1",
        user_id,
    )
    .fetch_one(db)
    .await
}

pub async fn insert_contact_access(
    db: impl PgExecutor<'_>,
    a: &ContactAccess<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO contact_access_log (id, employer_user_id, company_id, candidate_id, basis, basis_id,
                                         ip, user_agent)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        a.id,
        a.employer_user_id,
        a.company_id,
        a.candidate_id,
        a.basis,
        a.basis_id,
        a.ip,
        a.user_agent,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
