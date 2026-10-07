//! The candidate pool behind shortlists, bank search and category cards. Each specialization's
//! searchable candidates are loaded once into a [`SpecPool`] (compact scoring data only) and cached
//! by [`crate::cache::CandidateCache`]; a request filters it in memory and attaches its own text
//! closeness: by meaning where the need and the profile have vectors of one model, by keywords
//! otherwise. Display data is read for the final top-N only and kept in the same snapshot
//! ([`crate::cards`]).

use std::{hash::BuildHasher, pin::pin, sync::Arc};

use arc_swap::ArcSwapOption;
use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{
    Achievement, CandidatePool, CandidateSkill, CandidateSnapshot, FspSummary, SkillId,
    SkillSource, TextCloseness, semantic::Embeddings,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cache::Flight, cards::CachedCard, dto::parse_slugs, embeddings::NeedVector, error::ApiResult,
    state::AppState,
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
    /// The need's vector: closeness by meaning for every candidate with a vector of its model.
    pub meaning: Option<&'a NeedVector>,
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
    /// Profile vectors by index, `i8`-quantized (316 bytes for 384 dimensions); read only when a
    /// need with a vector is ranked, so they stay out of the hot scoring records.
    vectors: Embeddings,
    /// The model of each row's vector, as an index into `models`.
    vector_models: Box<[u8]>,
    models: Vec<Box<str>>,
    /// Closeness by meaning of each candidate to a need, per need vector; like `texts`.
    meaning: [ArcSwapOption<MeaningScores>; TEXT_SLOTS],
}

/// A row without a vector of the need's model: its closeness falls back to keywords.
const NO_VECTOR: f32 = f32::NAN;
const NO_MODEL: u8 = u8::MAX;

/// 0..1 closeness by meaning per snapshot index for one need vector; [`NO_VECTOR`] where missing.
struct MeaningScores {
    need: NeedVector,
    closeness: Box<[f32]>,
    /// Every row has a vector: the keyword ranks are not needed at all.
    complete: bool,
}

/// Raw `ts_rank_cd` per snapshot index for one query; 0 where the text does not match.
struct TextRanks {
    query: Box<str>,
    ranks: Box<[f32]>,
}

impl SpecPool {
    /// `vectors`: also load the profile vectors (embeddings on).
    pub async fn load(pg: &PgPool, spec: Specialization, vectors: bool) -> ApiResult<Self> {
        let started = std::time::Instant::now();
        let rows = db::pool::searchable_pool(pg, spec.slug()).await?;
        let mut pool = Self::from_rows(spec, rows, Utc::now().timestamp());
        if vectors {
            pool.attach_vectors(db::embeddings::pool_vectors(pg, spec.slug()).await?);
        }
        tracing::debug!(
            specialization = spec.slug(),
            candidates = pool.ids.len(),
            with_vectors = pool.vectors.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "candidate pool loaded"
        );
        Ok(pool)
    }

    /// Rows with a grade this build does not know are left out.
    fn from_rows(spec: Specialization, mut rows: Vec<db::pool::PoolCandidate>, now: i64) -> Self {
        rows.sort_unstable_by_key(|r| r.user_id);
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
                text_by_meaning: false,
            });
        }
        Self {
            pool: CandidatePool::from_snapshots(&snapshots),
            cards: ids.iter().map(|_| ArcSwapOption::empty()).collect(),
            vectors: Embeddings::default(),
            vector_models: vec![NO_MODEL; ids.len()].into_boxed_slice(),
            models: Vec::new(),
            ids,
            minors,
            texts: Default::default(),
            text_flights: Default::default(),
            meaning: Default::default(),
            hasher: std::hash::RandomState::new(),
        }
    }

    /// Attaches the stored profile vectors (ordered by id) to the snapshot's rows; vectors of
    /// candidates not in it are skipped. Up to 254 distinct models are told apart.
    fn attach_vectors(&mut self, rows: Vec<db::embeddings::StoredVector>) {
        let dim = rows.first().map_or(0, |r| r.embedding.as_slice().len());
        let mut table = Embeddings::with_capacity(dim, self.ids.len());
        let mut rows = rows.into_iter().peekable();
        for (i, id) in self.ids.iter().enumerate() {
            while rows.next_if(|r| r.id < *id).is_some() {}
            let Some(r) = rows.next_if(|r| r.id == *id) else {
                table.push(None);
                continue;
            };
            let model = match self.models.iter().position(|m| **m == *r.model) {
                Some(m) => m,
                None if self.models.len() < usize::from(NO_MODEL) => {
                    self.models.push(r.model.into_boxed_str());
                    self.models.len() - 1
                }
                None => {
                    table.push(None);
                    continue;
                }
            };
            self.vector_models[i] = model as u8;
            table.push(Some(r.embedding.as_slice()));
        }
        self.vectors = table;
    }

    /// Closeness by meaning of every row to the need, computed once per snapshot and need vector.
    fn meaning_scores(&self, need: &NeedVector) -> Arc<MeaningScores> {
        let i = self.hasher.hash_one(need.query.codes()) as usize % TEXT_SLOTS;
        let slot = &self.meaning[i];
        if let Some(cached) = slot.load_full().filter(|c| c.need == *need) {
            return cached;
        }
        let model = self.models.iter().position(|m| **m == *need.model);
        let closeness: Box<[f32]> = (0..self.ids.len())
            .map(|row| {
                model
                    .filter(|&m| self.vector_models.get(row) == Some(&(m as u8)))
                    .and_then(|_| self.vectors.cosine(row, &need.query))
                    .map_or(NO_VECTOR, |cos| need.calibration.closeness(cos))
            })
            .collect();
        let computed = Arc::new(MeaningScores {
            need: need.clone(),
            complete: closeness.iter().all(|c| !c.is_nan()),
            closeness,
        });
        slot.store(Some(Arc::clone(&computed)));
        computed
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

    fn filtered(
        &self,
        filter: &PoolFilter<'_>,
        ranks: Option<&TextRanks>,
        meaning: Option<&MeaningScores>,
    ) -> LoadedPool {
        let required: Vec<SkillId> = filter
            .required_skills
            .iter()
            .map(|s| SkillId::intern(s))
            .collect();
        let mut picks: Vec<(usize, f32)> = Vec::with_capacity(self.ids.len());
        let mut max_rank = 0f32;
        for i in 0..self.pool.len() {
            let Some(p) = self.pool.profile(i) else {
                continue;
            };
            if filter.grade.is_some_and(|g| p.category().grade != g)
                || (filter.adults_only && self.is_minor(i))
            {
                continue;
            }
            let rank = ranks.and_then(|r| r.ranks.get(i).copied()).unwrap_or(0.0);
            max_rank = max_rank.max(rank);
            if filter.fsp_only && !p.has_fsp() {
                continue;
            }
            if let (Some(max), Some(expect)) = (filter.max_expectation, p.salary_expectation())
                && i64::from(expect) > max
            {
                continue;
            }
            if !required.iter().all(|&s| self.pool.has_skill(i, s))
                || filter.only.is_some_and(|id| self.ids[i] != id)
            {
                continue;
            }
            picks.push((i, rank));
        }
        let ids = picks.iter().map(|&(i, _)| self.ids[i]).collect();
        let pool = self.pool.subset(picks.into_iter().map(|(i, rank)| {
            let by_meaning = meaning
                .and_then(|m| m.closeness.get(i).copied())
                .filter(|c| !c.is_nan());
            let text = match by_meaning {
                Some(c) => TextCloseness::meaning(c),
                None => TextCloseness::keywords(if max_rank > 0.0 { rank / max_rank } else { 0.0 }),
            };
            (i, text)
        }));
        LoadedPool { pool, ids }
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
    let meaning = filter.meaning.map(|need| snap.meaning_scores(need));
    let ranks = match filter.text.and_then(or_tsquery) {
        Some(query) if !meaning.as_ref().is_some_and(|m| m.complete) => {
            Some(snap.text_ranks(&state.pool, spec, query).await?)
        }
        _ => None,
    };
    Ok(snap.filtered(filter, ranks.as_deref(), meaning.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_791_374_400;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn row(n: u128) -> db::pool::PoolCandidate {
        db::pool::PoolCandidate {
            user_id: id(n),
            grade: "middle".into(),
            theta: None,
            salary_expectation_min: None,
            work_formats: Vec::new(),
            city: None,
            relocation: false,
            last_activity_at: "2026-10-01T10:00:00Z".parse().unwrap(),
            industries: Vec::new(),
            show_fsp: false,
            achievements: None,
            skills: vec!["go".into()],
            skill_sources: vec!["test".into()],
            minor: false,
        }
    }

    fn snapshot(rows: Vec<db::pool::PoolCandidate>) -> SpecPool {
        SpecPool::from_rows(Specialization::Backend, rows, NOW)
    }

    fn ranks(ranks: &[f32]) -> TextRanks {
        TextRanks {
            query: "go".into(),
            ranks: ranks.into(),
        }
    }

    #[test]
    fn snapshot_is_ordered_by_id_and_skips_unknown_grades() {
        let rows = vec![
            row(3),
            db::pool::PoolCandidate {
                grade: "guru".into(),
                ..row(2)
            },
            row(1),
        ];
        let snap = snapshot(rows);
        assert_eq!(snap.ids, [id(1), id(3)]);
        assert_eq!(snap.pool.id(1), Some(id(3).to_string().as_str()));
        let all = snap.filtered(&PoolFilter::default(), None, None);
        assert_eq!(all.ids, [id(1), id(3)]);
    }

    #[test]
    fn adults_only_drops_minors_across_bitset_words() {
        let minors = [2, 64, 65, 129];
        let rows: Vec<_> = (0..130u128)
            .rev()
            .map(|n| db::pool::PoolCandidate {
                minor: minors.contains(&n),
                ..row(n)
            })
            .collect();
        let snap = snapshot(rows);
        assert_eq!(
            snap.filtered(&PoolFilter::default(), None, None).ids.len(),
            130,
            "minors with the representative's consent are in the pool"
        );
        let adults = snap.filtered(
            &PoolFilter {
                adults_only: true,
                ..PoolFilter::default()
            },
            None,
            None,
        );
        assert_eq!(adults.ids.len(), 126);
        for n in minors {
            assert!(!adults.ids.contains(&id(n)), "{n}");
        }
        assert!(adults.ids.contains(&id(63)) && adults.ids.contains(&id(66)));
    }

    #[test]
    fn minor_bit_follows_the_kept_rows_after_a_skipped_one() {
        let rows = vec![
            db::pool::PoolCandidate {
                grade: "guru".into(),
                ..row(1)
            },
            db::pool::PoolCandidate {
                minor: true,
                ..row(2)
            },
            row(3),
        ];
        let adults = snapshot(rows).filtered(
            &PoolFilter {
                adults_only: true,
                ..PoolFilter::default()
            },
            None,
            None,
        );
        assert_eq!(adults.ids, [id(3)]);
    }

    #[test]
    fn grade_skills_expectation_and_only_filter_the_snapshot() {
        let rows = vec![
            db::pool::PoolCandidate {
                grade: "junior".into(),
                ..row(1)
            },
            db::pool::PoolCandidate {
                skills: vec!["go".into(), "postgresql".into()],
                skill_sources: vec!["test".into(), "declared".into()],
                salary_expectation_min: Some(150_000),
                ..row(2)
            },
            db::pool::PoolCandidate {
                salary_expectation_min: Some(250_000),
                ..row(3)
            },
            row(4),
        ];
        let snap = snapshot(rows);
        let pick = |filter: PoolFilter<'_>| snap.filtered(&filter, None, None).ids;
        assert_eq!(
            pick(PoolFilter {
                grade: Some(Grade::Middle),
                ..PoolFilter::default()
            }),
            [id(2), id(3), id(4)]
        );
        let required = ["postgresql".to_owned()];
        assert_eq!(
            pick(PoolFilter {
                required_skills: &required,
                ..PoolFilter::default()
            }),
            [id(2)]
        );
        assert_eq!(
            pick(PoolFilter {
                max_expectation: Some(150_000),
                ..PoolFilter::default()
            }),
            [id(1), id(2), id(4)],
            "above the budget is dropped, at it or unstated is kept"
        );
        assert_eq!(
            pick(PoolFilter {
                only: Some(id(3)),
                ..PoolFilter::default()
            }),
            [id(3)]
        );
        assert!(
            pick(PoolFilter {
                fsp_only: true,
                ..PoolFilter::default()
            })
            .is_empty(),
            "nobody here has FSP history"
        );
    }

    #[test]
    fn relevance_is_rank_over_the_best_rank_before_the_other_filters() {
        let rows = vec![
            row(1),
            db::pool::PoolCandidate {
                salary_expectation_min: Some(500_000),
                ..row(2)
            },
            row(3),
            db::pool::PoolCandidate {
                grade: "senior".into(),
                ..row(4)
            },
        ];
        let snap = snapshot(rows);
        let text = ranks(&[0.25, 1.0, 0.0, 2.0]);
        let relevance = |loaded: &LoadedPool| -> Vec<f32> {
            (0..loaded.pool.len())
                .map(|i| loaded.pool.profile(i).unwrap().text_relevance())
                .collect()
        };
        let all = snap.filtered(&PoolFilter::default(), Some(&text), None);
        assert_eq!(relevance(&all), [0.125, 0.5, 0.0, 1.0]);
        let middle_on_budget = snap.filtered(
            &PoolFilter {
                grade: Some(Grade::Middle),
                max_expectation: Some(200_000),
                ..PoolFilter::default()
            },
            Some(&text),
            None,
        );
        assert_eq!(middle_on_budget.ids, [id(1), id(3)]);
        assert_eq!(
            relevance(&middle_on_budget),
            [0.25, 0.0],
            "normalized by the best middle, even though the budget then drops them"
        );
        let unranked = snap.filtered(&PoolFilter::default(), None, None);
        assert!(relevance(&unranked).iter().all(|r| *r == 0.0));
    }

    #[test]
    fn tsquery_is_sanitized() {
        assert_eq!(
            or_tsquery("Go, PostgreSQL & Kafka!!! ':*").as_deref(),
            Some("postgresql | kafka")
        );
        assert_eq!(or_tsquery("a b"), None);
    }
}
