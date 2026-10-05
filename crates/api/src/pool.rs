//! Loads searchable candidates into the matching engine. Only the compact scoring data goes
//! into the pool; display data is fetched later for the final top-N only.

use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{
    Achievement, CandidatePool, CandidateSkill, CandidateSnapshot, FspSummary, SkillSource,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{dto::parse_slugs, error::ApiResult};

pub struct LoadedPool {
    pub pool: CandidatePool,
    pub ids: Vec<Uuid>,
}

#[derive(Default)]
pub struct PoolFilter<'a> {
    pub grade: Option<Grade>,
    pub text: Option<&'a str>,
    pub fsp_only: bool,
    pub max_expectation: Option<i64>,
    pub required_skills: &'a [String],
    /// «Только 18+»: a filter, never a ranking penalty.
    pub adults_only: bool,
}

pub fn source(s: &str) -> SkillSource {
    match s {
        "test" => SkillSource::Test,
        "fsp" => SkillSource::Fsp,
        "resume" => SkillSource::Resume,
        _ => SkillSource::Declared,
    }
}

pub fn achievements(json: Option<&serde_json::Value>) -> Vec<Achievement> {
    json.and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

pub fn fsp_summary(achievements: &[Achievement], spec: Specialization, now: i64) -> FspSummary {
    FspSummary::from(&matching::fsp_score(achievements, spec, now))
}

/// OR-query over the significant words of a text, safe for `to_tsquery` (letters and digits only).
pub fn or_tsquery(text: &str) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for w in text.split(|c: char| !c.is_alphanumeric()) {
        if w.chars().count() < 3 || words.len() >= 40 {
            continue;
        }
        let w = w.to_lowercase();
        if !words.contains(&w) {
            words.push(w);
        }
    }
    (!words.is_empty()).then(|| words.join(" | "))
}

pub fn unix(t: DateTime<Utc>) -> i64 {
    t.timestamp()
}

pub async fn load(
    pool: &PgPool,
    spec: Specialization,
    filter: &PoolFilter<'_>,
    now: i64,
) -> ApiResult<LoadedPool> {
    let text_query = filter.text.and_then(or_tsquery);
    let rows = db::pool::candidate_pool(
        pool,
        text_query.as_deref(),
        spec.slug(),
        filter.grade.map(Grade::slug),
        filter.adults_only,
    )
    .await?;
    let max_rank = rows.iter().map(|r| r.text_rank).fold(0.0f32, f32::max);
    let mut snapshots = Vec::with_capacity(rows.len());
    let mut ids = Vec::with_capacity(rows.len());
    for r in rows {
        let Some(grade) = r.grade.parse::<Grade>().ok() else {
            continue;
        };
        let skills: Vec<CandidateSkill> = r
            .skills
            .iter()
            .zip(&r.skill_sources)
            .map(|(slug, src)| CandidateSkill {
                slug: slug.clone(),
                source: source(src),
            })
            .collect();
        if !filter
            .required_skills
            .iter()
            .all(|req| skills.iter().any(|s| &s.slug == req))
        {
            continue;
        }
        let fsp = if r.show_fsp {
            fsp_summary(&achievements(r.achievements.as_ref()), spec, now)
        } else {
            FspSummary::default()
        };
        if filter.fsp_only && !fsp.has_history {
            continue;
        }
        if let (Some(max), Some(expect)) = (filter.max_expectation, r.salary_expectation_min)
            && expect > max
        {
            continue;
        }
        ids.push(r.user_id);
        snapshots.push(CandidateSnapshot {
            id: r.user_id.to_string(),
            category: Category::new(spec, grade),
            theta: r.theta.unwrap_or(grade.theta_cut()),
            skills,
            fsp,
            salary_expectation_min: r.salary_expectation_min,
            work_formats: parse_slugs::<WorkFormat>(&r.work_formats),
            city: r.city,
            relocation: r.relocation,
            last_activity_unix: unix(r.last_activity_at),
            industries: parse_slugs::<Industry>(&r.industries),
            text_relevance: if max_rank > 0.0 {
                f64::from(r.text_rank / max_rank)
            } else {
                0.0
            },
        });
    }
    Ok(LoadedPool {
        pool: CandidatePool::from_snapshots(&snapshots),
        ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tsquery_is_sanitized() {
        assert_eq!(
            or_tsquery("Go, PostgreSQL & Kafka!!! ':*").as_deref(),
            Some("postgresql | kafka")
        );
        assert_eq!(or_tsquery("a b"), None);
    }
}
