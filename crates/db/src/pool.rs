use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    proctoring::CardBadge,
    types::{InvitationStatus, SearchStatus},
};

#[derive(Debug, Clone)]
pub struct PoolCandidate {
    pub user_id: Uuid,
    pub grade: String,
    pub theta: Option<f64>,
    pub salary_expectation_min: Option<i64>,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub relocation: bool,
    pub last_activity_at: DateTime<Utc>,
    pub industries: Vec<String>,
    pub show_fsp: bool,
    pub achievements: Option<Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    /// Under 18 today; such a candidate is here only with the representative's consent.
    pub minor: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextRank {
    pub user_id: Uuid,
    pub rank: f32,
}

/// An employer's card of one candidate with the company's relation to them and the visibility
/// flags, as one row.
#[derive(Debug, Clone)]
pub struct CandidateCard {
    pub user_id: Uuid,
    pub public_code: String,
    pub full_name_enc: Option<Vec<u8>>,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub city: Option<String>,
    pub relocation: bool,
    pub work_formats: Vec<String>,
    pub salary_expectation_min: Option<i64>,
    pub about: String,
    pub experience_years: Option<f32>,
    pub roles: Vec<String>,
    pub soft_skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience: Value,
    pub search_status: SearchStatus,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub achievements: Option<Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    pub task_rating: Option<f32>,
    pub tasks_done: i64,
    /// Under 18 today; the birth date itself never leaves the database for employers.
    pub minor: bool,
    /// Visible, publication consent in force and, under 18, the representative's consent.
    pub published: bool,
    /// The company's latest invitation to the candidate.
    pub invitation_status: Option<InvitationStatus>,
    /// Applied to one of the company's vacancies and did not withdraw.
    pub applied: bool,
    /// A moderator confirmed the result of the test behind the current category; `None`
    /// without a category.
    pub result_confirmed: Option<bool>,
    /// Proctoring of the test behind the current category; `None` when it was not proctored.
    pub proctoring_camera: Option<bool>,
    pub proctoring_chunks: Option<i64>,
    pub tab_hidden: Option<i64>,
    pub window_blur: Option<i64>,
    pub fullscreen_exit: Option<i64>,
    pub paste: Option<i64>,
    pub camera_lost: Option<i64>,
    pub recording_failed: Option<i64>,
}

/// The columns of [`CandidateCard`] read on every display even when the rest of the card is
/// cached: visibility and privacy flags, which must never lag, the company's relation, and the
/// fields that change without a cache generation bump (activity time, age).
#[derive(Debug, Clone)]
pub struct CardLive {
    pub user_id: Uuid,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub search_status: SearchStatus,
    pub last_activity_at: DateTime<Utc>,
    pub minor: bool,
    pub published: bool,
    pub invitation_status: Option<InvitationStatus>,
    pub applied: bool,
}

impl CardLive {
    pub fn searchable(&self) -> bool {
        self.published && self.search_status != SearchStatus::Closed
    }
}

impl CandidateCard {
    /// Published and not closed to offers: the rule of [`searchable_pool`]. Derived here rather
    /// than in SQL, where a second reference to `published` evaluated its subqueries twice.
    pub fn searchable(&self) -> bool {
        self.published && self.search_status != SearchStatus::Closed
    }

    pub fn proctoring(&self) -> Option<CardBadge> {
        Some(CardBadge {
            candidate_id: self.user_id,
            camera: self.proctoring_camera?,
            chunks: self.proctoring_chunks.unwrap_or(0),
            tab_hidden: self.tab_hidden.unwrap_or(0),
            window_blur: self.window_blur.unwrap_or(0),
            fullscreen_exit: self.fullscreen_exit.unwrap_or(0),
            paste: self.paste.unwrap_or(0),
            camera_lost: self.camera_lost.unwrap_or(0),
            recording_failed: self.recording_failed.unwrap_or(0),
        })
    }
}

/// Searchable candidates of one specialization with everything the matching engine needs,
/// ordered by id. `skills`/`skill_sources` are parallel arrays; skills confirmed by FSP are left
/// out while the candidate hides FSP data, like the achievements themselves. The API caches the
/// result per specialization (`api::pool`), so it carries no per-request filters: grade,
/// «Только 18+» and the rest are applied to the cached snapshot.
///
/// Searchable means published (the `published_candidates` view: visible, holding an unrevoked
/// `pd_publication` grant and, while under 18, an unrevoked `guardian_consent` given by the
/// legal representative) and not closed. The grant checks repeat what `PUT /candidate/privacy`
/// enforces, so a stale `visible` flag can never publish a profile; both semi-joins read
/// `user_consents_document_idx`. Skills are aggregated per candidate in a lateral subquery:
/// grouping the whole joined set was the slowest step of the query at 10 000 candidates. sqlx
/// sees the view's columns as nullable, hence the `!` overrides.
pub async fn searchable_pool(
    db: impl PgExecutor<'_>,
    specialization: &str,
) -> Result<Vec<PoolCandidate>, sqlx::Error> {
    sqlx::query_as!(
        PoolCandidate,
        r#"SELECT c.user_id AS "user_id!", c.grade AS "grade!", c.theta, c.salary_expectation_min,
                  c.work_formats AS "work_formats!", c.city, c.relocation AS "relocation!",
                  c.last_activity_at AS "last_activity_at!", c.industries AS "industries!",
                  c.show_fsp AS "show_fsp!",
                  f.achievements AS "achievements?", sk.skills AS "skills!",
                  sk.sources AS "skill_sources!",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!"
           FROM published_candidates c
           JOIN users u ON u.id = c.user_id AND u.deleted_at IS NULL
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           CROSS JOIN LATERAL (
               SELECT coalesce(array_agg(s.skill ORDER BY s.skill, s.source), '{}') AS skills,
                      coalesce(array_agg(s.source ORDER BY s.skill, s.source), '{}') AS sources
               FROM candidate_skills s
               WHERE s.candidate_id = c.user_id AND (s.source <> 'fsp' OR c.show_fsp)
           ) sk
           WHERE c.search_status <> 'closed'
             AND c.specialization = $1
             AND c.grade IS NOT NULL
           ORDER BY c.user_id"#,
        specialization,
    )
    .fetch_all(db)
    .await
}

/// `ts_rank_cd` of each candidate's "about" against an OR-query, for the specialization's
/// candidates that match it at all (the rest rank 0), ordered by id. Visibility is not checked
/// here: ranks are only attached to candidates already in [`searchable_pool`].
pub async fn text_ranks(
    db: impl PgExecutor<'_>,
    specialization: &str,
    text_query: &str,
) -> Result<Vec<TextRank>, sqlx::Error> {
    sqlx::query_as!(
        TextRank,
        r#"SELECT c.user_id, ts_rank_cd(c.about_tsv, q)::real AS "rank!"
           FROM candidates c, to_tsquery('russian', $2) q
           WHERE c.specialization = $1 AND c.grade IS NOT NULL
             AND c.visible AND c.search_status <> 'closed'
             AND c.about_tsv @@ q
           ORDER BY c.user_id"#,
        specialization,
        text_query,
    )
    .fetch_all(db)
    .await
}

/// Cards of `ids` as `company_id` sees them, in one round trip. Nullability is spelled out:
/// inferred from the plan, it depends on the join order the planner picks for the data at hand. Skills and task stats are
/// aggregated per candidate in lateral subqueries (a group-by over the wide joined rows cost twice
/// as much); the company's relation and the proctoring of the latest grade assignment ride along.
/// The API keeps the rest of the card in memory (`api::cards`) and reads only [`card_live`] for
/// the cards it holds, so visibility is checked live on every display either way.
///
/// The ids go through a subquery so the planner cannot see them: with a visible one-element
/// array it kept re-planning this statement on every call (about 0.7 ms, four times the
/// execution) instead of switching to the cached generic plan, which suits any number of ids.
/// «Applied» is a scalar subquery with `LIMIT 1` rather than `EXISTS`: the planner turned the
/// `EXISTS` into a hashed scan of all the company's applications, which costs more than one
/// probe of `applications_candidate_idx` per card as soon as the company has a few thousand.
pub async fn candidate_cards(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    ids: &[Uuid],
) -> Result<Vec<CandidateCard>, sqlx::Error> {
    sqlx::query_as!(
        CandidateCard,
        r#"SELECT c.user_id AS "user_id!", c.public_code AS "public_code!", c.full_name_enc,
                  c.show_name AS "show_name!", c.show_fsp AS "show_fsp!",
                  c.show_experience AS "show_experience!", c.city, c.relocation AS "relocation!",
                  c.work_formats AS "work_formats!", c.salary_expectation_min, c.about AS "about!",
                  c.experience_years, c.roles AS "roles!", c.soft_skills AS "soft_skills!",
                  c.industries AS "industries!", c.experience AS "experience!",
                  c.search_status AS "search_status!: SearchStatus", c.specialization, c.grade, c.theta,
                  c.category_assigned_at, c.last_activity_at AS "last_activity_at!",
                  f.achievements AS "achievements?",
                  sk.skills AS "skills!", sk.sources AS "skill_sources!",
                  ta.task_rating, ta.tasks_done AS "tasks_done!",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!",
                  vis.published AS "published!",
                  (SELECT i.status FROM invitations i WHERE i.company_id = $1 AND i.candidate_id = c.user_id
                    ORDER BY i.created_at DESC LIMIT 1) AS "invitation_status?: InvitationStatus",
                  (SELECT true FROM applications a JOIN vacancies v ON v.id = a.vacancy_id
                    WHERE a.candidate_id = c.user_id AND v.company_id = $1
                      AND a.status <> 'withdrawn' LIMIT 1) IS NOT NULL AS "applied!",
                  latest.confirmed AS "result_confirmed?",
                  pb.camera AS "proctoring_camera?", pb.chunks AS "proctoring_chunks?",
                  pb.tab_hidden AS "tab_hidden?", pb.window_blur AS "window_blur?",
                  pb.fullscreen_exit AS "fullscreen_exit?", pb.paste AS "paste?",
                  pb.camera_lost AS "camera_lost?", pb.recording_failed AS "recording_failed?"
           FROM candidates c
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           CROSS JOIN LATERAL (
               SELECT coalesce(array_agg(s.skill ORDER BY s.skill, s.source), '{}') AS skills,
                      coalesce(array_agg(s.source ORDER BY s.skill, s.source), '{}') AS sources
               FROM candidate_skills s
               WHERE s.candidate_id = c.user_id AND (s.source <> 'fsp' OR c.show_fsp)
           ) sk
           CROSS JOIN LATERAL (
               SELECT (avg(a.rating) FILTER (WHERE a.status = 'reviewed'))::real AS task_rating,
                      count(*) FILTER (WHERE a.status IN ('submitted', 'reviewed')) AS tasks_done
               FROM task_assignments a
               WHERE a.candidate_id = c.user_id
           ) ta
           CROSS JOIN LATERAL (
               SELECT EXISTS (SELECT 1 FROM published_candidates p WHERE p.user_id = c.user_id) AS published
           ) vis
           LEFT JOIN LATERAL (
               SELECT g.session_id,
                      EXISTS (SELECT 1 FROM session_reviews r
                               WHERE r.session_id = g.session_id AND r.decision = 'confirm') AS confirmed
               FROM grade_assignments g
               WHERE g.candidate_id = c.user_id AND g.annulled_at IS NULL
               ORDER BY g.assigned_at DESC, g.id DESC
               LIMIT 1
           ) latest ON true
           LEFT JOIN LATERAL (
               SELECT p.camera,
                      (SELECT count(*) FROM proctoring_chunks pc WHERE pc.session_id = p.session_id) AS chunks,
                      count(e.kind) FILTER (WHERE e.kind = 'tab_hidden') AS tab_hidden,
                      count(e.kind) FILTER (WHERE e.kind = 'window_blur') AS window_blur,
                      count(e.kind) FILTER (WHERE e.kind = 'fullscreen_exit') AS fullscreen_exit,
                      count(e.kind) FILTER (WHERE e.kind = 'paste') AS paste,
                      count(e.kind) FILTER (WHERE e.kind = 'camera_lost') AS camera_lost,
                      count(e.kind) FILTER (WHERE e.kind = 'recording_failed') AS recording_failed
               FROM proctoring_sessions p
               LEFT JOIN proctoring_events e ON e.session_id = p.session_id
               WHERE p.session_id = latest.session_id
               GROUP BY p.session_id
           ) pb ON true
           WHERE c.user_id = ANY ((SELECT $2::uuid[])::uuid[])"#,
        company_id,
        ids,
    )
    .fetch_all(db)
    .await
}

/// The [`CardLive`] columns of `ids` as `company_id` sees them: the same expressions as in
/// [`candidate_cards`], without the aggregates over skills, tasks and proctoring that make up
/// most of its cost. The ids go through a subquery for the same reason.
pub async fn card_live(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    ids: &[Uuid],
) -> Result<Vec<CardLive>, sqlx::Error> {
    sqlx::query_as!(
        CardLive,
        r#"SELECT c.user_id AS "user_id!", c.show_name AS "show_name!", c.show_fsp AS "show_fsp!",
                  c.show_experience AS "show_experience!",
                  c.search_status AS "search_status!: SearchStatus",
                  c.last_activity_at AS "last_activity_at!",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!",
                  EXISTS (SELECT 1 FROM published_candidates p WHERE p.user_id = c.user_id) AS "published!",
                  (SELECT i.status FROM invitations i WHERE i.company_id = $1 AND i.candidate_id = c.user_id
                    ORDER BY i.created_at DESC LIMIT 1) AS "invitation_status?: InvitationStatus",
                  (SELECT true FROM applications a JOIN vacancies v ON v.id = a.vacancy_id
                    WHERE a.candidate_id = c.user_id AND v.company_id = $1
                      AND a.status <> 'withdrawn' LIMIT 1) IS NOT NULL AS "applied!"
           FROM candidates c
           WHERE c.user_id = ANY ((SELECT $2::uuid[])::uuid[])"#,
        company_id,
        ids,
    )
    .fetch_all(db)
    .await
}
