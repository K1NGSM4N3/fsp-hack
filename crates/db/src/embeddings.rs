//! Sentence embeddings of profiles, needs and vacancies and their work queue (migration 0017).

use chrono::{DateTime, Utc};
pub use pgvector::Vector;
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::EmbeddingKind;

#[derive(Debug, Clone)]
pub struct Job {
    pub kind: EmbeddingKind,
    pub entity_id: Uuid,
    /// The version of the job this worker took: deleted only if it is still the same.
    pub enqueued_at: DateTime<Utc>,
    /// Including this claim.
    pub attempts: i32,
}

/// Takes up to `limit` due jobs, oldest first, for `lease_secs`. `SKIP LOCKED`: rows another
/// replica is claiming right now are left to it.
pub async fn claim_jobs(
    db: impl PgExecutor<'_>,
    limit: i64,
    lease_secs: f64,
) -> Result<Vec<Job>, sqlx::Error> {
    sqlx::query_as!(
        Job,
        r#"UPDATE embedding_jobs j
           SET claimed_until = now() + make_interval(secs => $2), attempts = j.attempts + 1
           FROM (SELECT kind, entity_id FROM embedding_jobs
                 WHERE claimed_until IS NULL OR claimed_until < now()
                 ORDER BY enqueued_at
                 LIMIT $1
                 FOR UPDATE SKIP LOCKED) due
           WHERE j.kind = due.kind AND j.entity_id = due.entity_id
           RETURNING j.kind AS "kind: EmbeddingKind", j.entity_id, j.enqueued_at, j.attempts"#,
        limit,
        lease_secs,
    )
    .fetch_all(db)
    .await
}

/// Deletes the done jobs of one kind, each only if nobody re-enqueued it while it was worked on.
pub async fn finish_jobs(
    db: impl PgExecutor<'_>,
    kind: EmbeddingKind,
    ids: &[Uuid],
    enqueued: &[DateTime<Utc>],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM embedding_jobs j
         USING UNNEST($2::uuid[], $3::timestamptz[]) AS d(entity_id, enqueued_at)
         WHERE j.kind = $1 AND j.entity_id = d.entity_id AND j.enqueued_at = d.enqueued_at",
        kind as EmbeddingKind,
        ids,
        enqueued,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Hands jobs back to be retried no earlier than `delay_secs` from now.
pub async fn retry_jobs(
    db: impl PgExecutor<'_>,
    kind: EmbeddingKind,
    ids: &[Uuid],
    delay_secs: f64,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE embedding_jobs SET claimed_until = now() + make_interval(secs => $3)
         WHERE kind = $1 AND entity_id = ANY($2)",
        kind as EmbeddingKind,
        ids,
        delay_secs,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Jobs waiting, for the `embedding_jobs_pending` gauge.
pub async fn pending_jobs(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM embedding_jobs"#)
        .fetch_one(db)
        .await
}

/// Puts every entity whose vector came from another model back in the queue (a model change).
pub async fn enqueue_stale(db: impl PgExecutor<'_>, model: &str) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO embedding_jobs (kind, entity_id)
         SELECT 'candidate'::embedding_kind, candidate_id FROM candidate_embeddings WHERE model <> $1
         UNION ALL
         SELECT 'need'::embedding_kind, need_id FROM need_embeddings WHERE model <> $1
         UNION ALL
         SELECT 'vacancy'::embedding_kind, vacancy_id FROM vacancy_embeddings WHERE model <> $1
         ON CONFLICT (kind, entity_id) DO NOTHING",
        model,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// What a profile vector is computed from, with the stored vector's model and text hash.
#[derive(Debug, Clone)]
pub struct CandidateText {
    pub user_id: Uuid,
    pub about: String,
    pub roles: Vec<String>,
    /// `[{company, position, from, to, description}]`
    pub experience: Value,
    pub model: Option<String>,
    pub text_hash: Option<i64>,
}

pub async fn candidate_texts(
    db: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<CandidateText>, sqlx::Error> {
    sqlx::query_as!(
        CandidateText,
        r#"SELECT c.user_id, c.about, c.roles, c.experience,
                  e.model AS "model?", e.text_hash AS "text_hash?"
           FROM candidates c
           LEFT JOIN candidate_embeddings e ON e.candidate_id = c.user_id
           WHERE c.user_id = ANY($1)"#,
        ids,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct PostingText {
    pub id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    /// Vacancies: published now. Needs: always `false`.
    pub published: bool,
    pub model: Option<String>,
    pub text_hash: Option<i64>,
}

pub async fn need_texts(
    db: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<PostingText>, sqlx::Error> {
    sqlx::query_as!(
        PostingText,
        r#"SELECT n.id, n.company_id, n.title, n.description, false AS "published!",
                  e.model AS "model?", e.text_hash AS "text_hash?"
           FROM hiring_needs n
           LEFT JOIN need_embeddings e ON e.need_id = n.id
           WHERE n.id = ANY($1)"#,
        ids,
    )
    .fetch_all(db)
    .await
}

pub async fn vacancy_texts(
    db: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<PostingText>, sqlx::Error> {
    sqlx::query_as!(
        PostingText,
        r#"SELECT v.id, v.company_id, v.title, v.description,
                  (v.status = 'published') AS "published!",
                  e.model AS "model?", e.text_hash AS "text_hash?"
           FROM vacancies v
           LEFT JOIN vacancy_embeddings e ON e.vacancy_id = v.id
           WHERE v.id = ANY($1)"#,
        ids,
    )
    .fetch_all(db)
    .await
}

pub async fn upsert_candidate_embedding(
    db: impl PgExecutor<'_>,
    id: Uuid,
    model: &str,
    text_hash: i64,
    embedding: &Vector,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO candidate_embeddings (candidate_id, model, text_hash, embedding)
         SELECT $1, $2, $3, $4 WHERE EXISTS (SELECT 1 FROM candidates WHERE user_id = $1)
         ON CONFLICT (candidate_id) DO UPDATE
         SET model = EXCLUDED.model, text_hash = EXCLUDED.text_hash,
             embedding = EXCLUDED.embedding, embedded_at = now()",
        id,
        model,
        text_hash,
        embedding as &Vector,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn upsert_need_embedding(
    db: impl PgExecutor<'_>,
    id: Uuid,
    model: &str,
    text_hash: i64,
    embedding: &Vector,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO need_embeddings (need_id, model, text_hash, embedding)
         SELECT $1, $2, $3, $4 WHERE EXISTS (SELECT 1 FROM hiring_needs WHERE id = $1)
         ON CONFLICT (need_id) DO UPDATE
         SET model = EXCLUDED.model, text_hash = EXCLUDED.text_hash,
             embedding = EXCLUDED.embedding, embedded_at = now()",
        id,
        model,
        text_hash,
        embedding as &Vector,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn upsert_vacancy_embedding(
    db: impl PgExecutor<'_>,
    id: Uuid,
    model: &str,
    text_hash: i64,
    embedding: &Vector,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO vacancy_embeddings (vacancy_id, model, text_hash, embedding)
         SELECT $1, $2, $3, $4 WHERE EXISTS (SELECT 1 FROM vacancies WHERE id = $1)
         ON CONFLICT (vacancy_id) DO UPDATE
         SET model = EXCLUDED.model, text_hash = EXCLUDED.text_hash,
             embedding = EXCLUDED.embedding, embedded_at = now()",
        id,
        model,
        text_hash,
        embedding as &Vector,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn delete_candidate_embedding(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM candidate_embeddings WHERE candidate_id = $1",
        id
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

#[derive(Debug, Clone)]
pub struct StoredVector {
    pub id: Uuid,
    pub model: String,
    pub embedding: Vector,
}

/// Profile vectors of the specialization's candidates with a category, ordered by id. Visibility
/// is not checked: they are only attached to candidates already in `pool::searchable_pool`.
pub async fn pool_vectors(
    db: impl PgExecutor<'_>,
    specialization: &str,
) -> Result<Vec<StoredVector>, sqlx::Error> {
    sqlx::query_as!(
        StoredVector,
        r#"SELECT e.candidate_id AS id, e.model, e.embedding AS "embedding: Vector"
           FROM candidate_embeddings e
           JOIN candidates c ON c.user_id = e.candidate_id
           WHERE c.specialization = $1 AND c.grade IS NOT NULL
           ORDER BY e.candidate_id"#,
        specialization,
    )
    .fetch_all(db)
    .await
}

pub async fn need_vector(
    db: impl PgExecutor<'_>,
    need_id: Uuid,
) -> Result<Option<StoredVector>, sqlx::Error> {
    sqlx::query_as!(
        StoredVector,
        r#"SELECT need_id AS id, model, embedding AS "embedding: Vector"
           FROM need_embeddings WHERE need_id = $1"#,
        need_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn vacancy_vector(
    db: impl PgExecutor<'_>,
    vacancy_id: Uuid,
) -> Result<Option<StoredVector>, sqlx::Error> {
    sqlx::query_as!(
        StoredVector,
        r#"SELECT vacancy_id AS id, model, embedding AS "embedding: Vector"
           FROM vacancy_embeddings WHERE vacancy_id = $1"#,
        vacancy_id,
    )
    .fetch_optional(db)
    .await
}

/// A published vacancy close to a given vector, with its cosine similarity.
#[derive(Debug, Clone)]
pub struct NearVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub specialization: String,
    pub similarity: f64,
}

/// Lets the HNSW scan of this transaction go on past the first `hnsw.ef_search` rows when the
/// filters drop them (pgvector ≥ 0.8): most rows of `vacancy_embeddings` may be drafts, closed
/// vacancies or the same company's. Results can come slightly out of order; callers re-sort.
pub async fn relaxed_scan(db: impl PgExecutor<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!("SET LOCAL hnsw.iterative_scan = relaxed_order")
        .execute(db)
        .await
        .map(|_| ())
}

/// The published vacancies nearest to a vector of the same model: optionally only other
/// companies' ones and only those published within `within_days`.
#[allow(clippy::too_many_arguments)]
pub async fn nearest_published(
    db: impl PgExecutor<'_>,
    embedding: &Vector,
    model: &str,
    exclude_id: Uuid,
    other_than_company: Option<Uuid>,
    within_days: Option<i32>,
    limit: i64,
) -> Result<Vec<NearVacancy>, sqlx::Error> {
    sqlx::query_as!(
        NearVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title, v.specialization,
                  1 - (e.embedding <=> $1) AS "similarity!"
           FROM vacancy_embeddings e
           JOIN vacancies v ON v.id = e.vacancy_id
           JOIN companies co ON co.id = v.company_id
           WHERE e.model = $2 AND v.status = 'published' AND v.id <> $3
             AND ($4::uuid IS NULL OR v.company_id <> $4)
             AND ($5::int IS NULL OR v.published_at > now() - make_interval(days => $5))
           ORDER BY e.embedding <=> $1
           LIMIT $6"#,
        embedding as &Vector,
        model,
        exclude_id,
        other_than_company,
        within_days,
        limit,
    )
    .fetch_all(db)
    .await
}

/// Without vectors: published vacancies of the same specialization sharing the most skills with
/// the given one, then the most recent.
pub async fn similar_by_skills(
    db: impl PgExecutor<'_>,
    vacancy_id: Uuid,
    limit: i64,
) -> Result<Vec<(Uuid, f64)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT v.id,
                  (SELECT count(*) FROM unnest(v.skills) s WHERE s = ANY(q.skills))::float8
                    / greatest(cardinality(q.skills), 1) AS "similarity!"
           FROM vacancies q
           JOIN vacancies v ON v.specialization = q.specialization AND v.id <> q.id
                            AND v.status = 'published'
           WHERE q.id = $1
           ORDER BY 2 DESC, v.published_at DESC
           LIMIT $2"#,
        vacancy_id,
        limit,
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|r| (r.id, r.similarity)).collect())
}
