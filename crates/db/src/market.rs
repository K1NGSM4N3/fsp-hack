use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CategoryOverview {
    pub specialization: String,
    pub grade: String,
    pub candidates: i64,
    pub with_fsp: i64,
    pub actively_looking: i64,
    pub median_expectation: Option<i64>,
}

/// Anonymous demand in one category, the same for every candidate in it.
#[derive(Debug, Clone, Copy)]
pub struct CategoryDemand {
    pub invitations_in_category: i64,
    pub open_needs: i64,
    pub peers: i64,
    pub median_salary_min: Option<i64>,
    pub median_salary_max: Option<i64>,
}

pub async fn category_overview(
    db: impl PgExecutor<'_>,
) -> Result<Vec<CategoryOverview>, sqlx::Error> {
    sqlx::query_as!(
        CategoryOverview,
        r#"SELECT c.specialization AS "specialization!", c.grade AS "grade!",
                  count(*) AS "candidates!",
                  count(*) FILTER (WHERE f.has_achievements AND c.show_fsp) AS "with_fsp!",
                  count(*) FILTER (WHERE c.search_status = 'active') AS "actively_looking!",
                  (percentile_cont(0.5) WITHIN GROUP (ORDER BY c.salary_expectation_min))::bigint
                      AS median_expectation
           FROM candidates c
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           WHERE c.visible AND c.search_status <> 'closed'
             AND c.specialization IS NOT NULL AND c.grade IS NOT NULL
             AND EXISTS (SELECT 1 FROM user_consents uc
                          JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                          WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)
             AND (c.birth_date IS NULL OR c.birth_date + interval '18 years' <= current_date
                  OR EXISTS (SELECT 1 FROM user_consents gc
                              JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                              WHERE gc.user_id = c.user_id AND gc.revoked_at IS NULL))
           GROUP BY c.specialization, c.grade"#,
    )
    .fetch_all(db)
    .await
}

/// Companies that opened the candidate's profile in the last 30 days.
pub async fn companies_viewed(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(DISTINCT pv.company_id) AS "count!" FROM profile_views pv
           WHERE pv.candidate_id = $1 AND pv.viewed_on >= current_date - 30"#,
        candidate_id,
    )
    .fetch_one(db)
    .await
}

pub async fn category_demand(
    db: impl PgExecutor<'_>,
    specialization: &str,
    grade: &str,
) -> Result<CategoryDemand, sqlx::Error> {
    sqlx::query_as!(
        CategoryDemand,
        r#"SELECT
               (SELECT count(*) FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $1 AND ic.grade = $2
                   AND i.created_at >= now() - interval '30 days') AS "invitations_in_category!",
               (SELECT count(*) FROM hiring_needs n
                 WHERE n.status = 'active' AND n.specialization = $1 AND $2 = ANY (n.grades)) AS "open_needs!",
               (SELECT count(*) FROM candidates oc
                 WHERE oc.specialization = $1 AND oc.grade = $2 AND oc.visible
                   AND oc.search_status <> 'closed'
                   AND EXISTS (SELECT 1 FROM user_consents uc
                                JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                                WHERE uc.user_id = oc.user_id AND uc.revoked_at IS NULL)
                   AND (oc.birth_date IS NULL OR oc.birth_date + interval '18 years' <= current_date
                        OR EXISTS (SELECT 1 FROM user_consents gc
                                    JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                                    WHERE gc.user_id = oc.user_id AND gc.revoked_at IS NULL))) AS "peers!",
               (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_min))::bigint
                  FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $1 AND ic.grade = $2
                   AND i.created_at >= now() - interval '90 days') AS median_salary_min,
               (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_max))::bigint
                  FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $1 AND ic.grade = $2
                   AND i.created_at >= now() - interval '90 days') AS median_salary_max"#,
        specialization,
        grade,
    )
    .fetch_one(db)
    .await
}

pub async fn record_profile_view(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO profile_views (company_id, candidate_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        company_id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
