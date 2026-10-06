//! The candidate pool behind shortlists, bank search and category cards. Each specialization's
//! searchable candidates are loaded once into a [`SpecPool`] (compact scoring data only) and cached
//! by [`crate::cache::CandidateCache`]; a request filters it in memory and attaches its own text
//! relevance. Display data is read for the final top-N only and kept in the same snapshot
//! ([`crate::cards`]).

use std::{hash::BuildHasher, pin::pin, sync::Arc};

use arc_swap::ArcSwapOption;
use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{
    Achievement, CandidatePool, CandidateSkill, CandidateSnapshot, FspSummary, SkillId, SkillSource,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::Flight, cards::CachedCard, dto::parse_slugs, error::ApiResult, state::AppState,
};

/// Distinct need texts whose ranks a snapshot keeps; a direct-mapped table, so a new text evicts
/// the one in its slot.
const TEXT_SLOTS: usize = 8;

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
    /// Keep only this candidate (one profile's explanation); text relevance is still normalized
    /// over the whole filtered pool.
    pub only: Option<Uuid>,
}

/// Searchable candidates of one specialization as of one generation, ordered by id.
pub struct SpecPool {
    pool: CandidatePool,
    ids: Vec<Uuid>,
    /// Bit per candidate: under 18 when loaded (present only with the representative's consent).
    minors: Vec<u64>,
    texts: [ArcSwapOption<TextRanks>; TEXT_SLOTS],
    /// One ranking query per slot at a time: right after a reload every request of the need
    /// misses the same slot.
    text_flights: [Flight; TEXT_SLOTS],
    hasher: std::hash::RandomState,
    /// Employer card of each candidate, by index, filled on first display: cold data, kept apart
    /// from the scoring pool and dropped with the snapshot.
    cards: Box<[ArcSwapOption<CachedCard>]>,
}

/// Raw `ts_rank_cd` per snapshot index for one query; 0 where the text does not match.
struct TextRanks {
    query: Box<str>,
    ranks: Box<[f32]>,
}

impl SpecPool {
    pub async fn load(pg: &PgPool, spec: Specialization) -> ApiResult<Self> {
        let started = std::time::Instant::now();
        let mut rows = db::pool::searchable_pool(pg, spec.slug()).await?;
        rows.sort_unstable_by_key(|r| r.user_id);
        let now = Utc::now().timestamp();
        let mut snapshots = Vec::with_capacity(rows.len());
        let mut ids = Vec::with_capacity(rows.len());
        let mut minors = vec![0u64; rows.len().div_ceil(64)];
        for r in rows {
            let Ok(grade) = r.grade.parse::<Grade>() else {
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
            let fsp = if r.show_fsp {
                fsp_summary(&achievements(r.achievements.as_ref()), spec, now)
            } else {
                FspSummary::default()
            };
            if r.minor {
                minors[ids.len() / 64] |= 1 << (ids.len() % 64);
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
                text_relevance: 0.0,
            });
        }
        tracing::debug!(
            specialization = spec.slug(),
            candidates = ids.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "candidate pool loaded"
        );
        Ok(Self {
            pool: CandidatePool::from_snapshots(&snapshots),
            cards: ids.iter().map(|_| ArcSwapOption::empty()).collect(),
            ids,
            minors,
            texts: Default::default(),
            text_flights: Default::default(),
            hasher: std::hash::RandomState::new(),
        })
    }

    pub fn card(&self, id: Uuid) -> Option<Arc<CachedCard>> {
        let i = self.ids.binary_search(&id).ok()?;
        self.cards.get(i)?.load_full()
    }

    /// Remembers a card of a candidate in this snapshot; others are not kept.
    pub fn keep_card(&self, id: Uuid, card: impl FnOnce() -> CachedCard) {
        if let Ok(i) = self.ids.binary_search(&id)
            && let Some(slot) = self.cards.get(i)
        {
            slot.store(Some(Arc::new(card())));
        }
    }

    fn is_minor(&self, index: usize) -> bool {
        self.minors
            .get(index / 64)
            .is_some_and(|w| w & (1 << (index % 64)) != 0)
    }

    /// Ranks of one need text, computed once per snapshot: `ts_rank_cd` over a long OR-query
    /// costs about as much as loading the snapshot itself.
    async fn text_ranks(
        &self,
        pg: &PgPool,
        spec: Specialization,
        query: String,
    ) -> ApiResult<Arc<TextRanks>> {
        let i = self.hasher.hash_one(&query) as usize % TEXT_SLOTS;
        let (slot, flight) = (&self.texts[i], &self.text_flights[i]);
        let cached = || slot.load_full().filter(|c| *c.query == *query);
        loop {
            if let Some(ranks) = cached() {
                return Ok(ranks);
            }
            let mut landed = pin!(flight.landed());
            landed.as_mut().enable();
            if let Some(_lead) = flight.lead() {
                if let Some(ranks) = cached() {
                    return Ok(ranks);
                }
                let mut ranks = vec![0f32; self.ids.len()];
                for r in db::pool::text_ranks(pg, spec.slug(), &query).await? {
                    if let Ok(i) = self.ids.binary_search(&r.user_id) {
                        ranks[i] = r.rank;
                    }
                }
                let computed = Arc::new(TextRanks {
                    query: query.into_boxed_str(),
                    ranks: ranks.into_boxed_slice(),
                });
                slot.store(Some(Arc::clone(&computed)));
                return Ok(computed);
            }
            if let Some(ranks) = cached() {
                return Ok(ranks);
            }
            landed.await;
        }
    }
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

/// The specialization's cached pool narrowed by the request's filters. Text relevance is
/// normalized to the best match among candidates of the requested grade and age, before the
/// remaining filters, as it was when the database ranked the filtered rows.
pub async fn load(
    state: &AppState,
    spec: Specialization,
    filter: &PoolFilter<'_>,
) -> ApiResult<LoadedPool> {
    let cached = state.candidates.pool(&state.pool, spec).await?;
    let snap = &cached.value;
    let ranks = match filter.text.and_then(or_tsquery) {
        Some(query) => Some(snap.text_ranks(&state.pool, spec, query).await?),
        None => None,
    };
    let required: Vec<SkillId> = filter
        .required_skills
        .iter()
        .map(|s| SkillId::intern(s))
        .collect();
    let mut picks: Vec<(usize, f32)> = Vec::with_capacity(snap.ids.len());
    let mut max_rank = 0f32;
    for i in 0..snap.pool.len() {
        let Some(p) = snap.pool.profile(i) else {
            continue;
        };
        if filter.grade.is_some_and(|g| p.category().grade != g)
            || (filter.adults_only && snap.is_minor(i))
        {
            continue;
        }
        let rank = ranks
            .as_ref()
            .and_then(|r| r.ranks.get(i).copied())
            .unwrap_or(0.0);
        max_rank = max_rank.max(rank);
        if filter.fsp_only && !p.has_fsp() {
            continue;
        }
        if let (Some(max), Some(expect)) = (filter.max_expectation, p.salary_expectation())
            && i64::from(expect) > max
        {
            continue;
        }
        if !required.iter().all(|&s| snap.pool.has_skill(i, s))
            || filter.only.is_some_and(|id| snap.ids[i] != id)
        {
            continue;
        }
        picks.push((i, rank));
    }
    let ids = picks.iter().map(|&(i, _)| snap.ids[i]).collect();
    let pool = snap.pool.subset(picks.into_iter().map(|(i, rank)| {
        let relevance = if max_rank > 0.0 { rank / max_rank } else { 0.0 };
        (i, relevance)
    }));
    Ok(LoadedPool { pool, ids })
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
