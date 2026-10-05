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

#[derive(Debug, Clone, Copy)]
pub struct CandidateMarket {
    pub companies_viewed_you: i64,
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
           GROUP BY c.specialization, c.grade"#,
    )
    .fetch_all(db)
    .await
}

pub async fn candidate_market(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    specialization: &str,
    grade: &str,
) -> Result<CandidateMarket, sqlx::Error> {
    sqlx::query_as!(
        CandidateMarket,
        r#"SELECT
               (SELECT count(DISTINCT pv.company_id) FROM profile_views pv
                 WHERE pv.candidate_id = $1 AND pv.viewed_on >= current_date - 30) AS "companies_viewed_you!",
               (SELECT count(*) FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $2 AND ic.grade = $3
                   AND i.created_at >= now() - interval '30 days') AS "invitations_in_category!",
               (SELECT count(*) FROM hiring_needs n
                 WHERE n.status = 'active' AND n.specialization = $2 AND $3 = ANY (n.grades)) AS "open_needs!",
               (SELECT count(*) FROM candidates oc
                 WHERE oc.specialization = $2 AND oc.grade = $3 AND oc.visible
                   AND oc.search_status <> 'closed'
                   AND EXISTS (SELECT 1 FROM user_consents uc
                                JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                                WHERE uc.user_id = oc.user_id AND uc.revoked_at IS NULL)) AS "peers!",
               (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_min))::bigint
                  FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $2 AND ic.grade = $3
                   AND i.created_at >= now() - interval '90 days') AS median_salary_min,
               (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_max))::bigint
                  FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
                 WHERE ic.specialization = $2 AND ic.grade = $3
                   AND i.created_at >= now() - interval '90 days') AS median_salary_max"#,
        candidate_id,
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
