//! Candidate data for matching. [`CandidateSnapshot`] is the owned API/DB-facing form;
//! [`CandidatePool`] stores many candidates as compact hot records (interned skill ids,
//! bitsets, `f32`) plus a separate cold part (ids, texts) read only when explaining top-N.

use std::sync::Arc;

use catalog::{Category, Grade, Industry, WorkFormat};
use serde::{Deserialize, Serialize};

use crate::fsp::{FspScore, NO_FSP_DATA_RU};
use crate::lexicon;
use crate::semantic::TextCloseness;
use crate::skills::SkillId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    Test,
    Fsp,
    Resume,
    Declared,
}

impl SkillSource {
    /// Verified by our test or the FSP registry > parsed from a resume > only declared (section 6.3).
    pub const fn confidence(self) -> f32 {
        match self {
            SkillSource::Test | SkillSource::Fsp => 1.0,
            SkillSource::Resume => 0.7,
            SkillSource::Declared => 0.5,
        }
    }

    pub const fn name_ru(self) -> &'static str {
        match self {
            SkillSource::Test => "подтверждён тестом",
            SkillSource::Fsp => "подтверждён ФСП",
            SkillSource::Resume => "из резюме",
            SkillSource::Declared => "заявлен",
        }
    }

    pub const fn is_verified(self) -> bool {
        matches!(self, SkillSource::Test | SkillSource::Fsp)
    }

    /// Which source a skill listed under several keeps: our own test before the FSP registry
    /// (same confidence), then the resume, then the declaration.
    const fn precedence(self) -> u8 {
        match self {
            SkillSource::Test => 3,
            SkillSource::Fsp => 2,
            SkillSource::Resume => 1,
            SkillSource::Declared => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CandidateSkill {
    /// Canonical slug; aliases and unknown names are accepted and interned.
    pub slug: String,
    pub source: SkillSource,
}

/// The part of [`FspScore`] matching needs; store it next to the candidate row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FspSummary {
    pub score: f32,
    pub has_history: bool,
    pub summary_ru: String,
}

impl Default for FspSummary {
    fn default() -> Self {
        FspSummary {
            score: 0.0,
            has_history: false,
            summary_ru: NO_FSP_DATA_RU.to_owned(),
        }
    }
}

impl From<&FspScore> for FspSummary {
    fn from(s: &FspScore) -> Self {
        FspSummary {
            score: s.score,
            has_history: s.has_history,
            summary_ru: s.summary_ru.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CandidateSnapshot {
    pub id: String,
    /// Current category from the test (M-1: never from the resume).
    pub category: Category,
    /// Ability estimate on the specialization's IRT scale.
    pub theta: f64,
    #[serde(default)]
    pub skills: Vec<CandidateSkill>,
    #[serde(default)]
    pub fsp: FspSummary,
    #[serde(default)]
    pub salary_expectation_min: Option<i64>,
    /// Empty = any format.
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub relocation: bool,
    /// Last test, task or profile update; `0` = unknown.
    pub last_activity_unix: i64,
    #[serde(default)]
    pub industries: Vec<Industry>,
    /// 0..1 closeness of the need description and the candidate's profile text, computed outside
    /// (embeddings, or `ts_rank` normalized by the best candidate); 0 when unavailable.
    #[serde(default)]
    pub text_relevance: f64,
    /// `text_relevance` is a closeness by meaning (embeddings), not by keywords.
    #[serde(default)]
    pub text_by_meaning: bool,
}

pub(crate) trait Flag: Copy {
    fn bit(self) -> u16;
}

impl Flag for WorkFormat {
    fn bit(self) -> u16 {
        1 << self as u16
    }
}

impl Flag for Industry {
    fn bit(self) -> u16 {
        1 << self as u16
    }
}

impl Flag for Grade {
    fn bit(self) -> u16 {
        1 << self as u16
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Bits(u16);

impl Bits {
    pub(crate) fn of<T: Flag>(items: impl IntoIterator<Item = T>) -> Bits {
        Bits(items.into_iter().fold(0, |acc, t| acc | t.bit()))
    }

    pub(crate) fn all_formats() -> Bits {
        Bits::of(WorkFormat::ALL.iter().copied())
    }

    pub(crate) fn has<T: Flag>(self, t: T) -> bool {
        self.0 & t.bit() != 0
    }

    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn and(self, other: Bits) -> Bits {
        Bits(self.0 & other.0)
    }

    pub(crate) fn or_if_empty(self, fallback: Bits) -> Bits {
        if self.is_empty() { fallback } else { self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SkillHit {
    pub id: SkillId,
    pub source: SkillSource,
}

/// Hot per-candidate record (~40 bytes, `Copy`); skills live in the pool's shared array.
#[derive(Debug, Clone, Copy)]
pub struct CandidateProfile {
    pub(crate) last_activity_unix: i64,
    pub(crate) theta: f32,
    pub(crate) fsp: f32,
    pub(crate) text_relevance: f32,
    /// Rubles; 0 = not specified.
    pub(crate) salary_min: u32,
    pub(crate) city: u32,
    pub(crate) skills_start: u32,
    /// Index of the candidate's cold record; survives [`CandidatePool::subset`].
    pub(crate) origin: u32,
    pub(crate) skills_len: u16,
    pub(crate) industries: Bits,
    pub(crate) formats: Bits,
    pub(crate) category: Category,
    pub(crate) relocation: bool,
    pub(crate) has_fsp: bool,
    pub(crate) text_by_meaning: bool,
}

impl CandidateProfile {
    pub fn category(&self) -> Category {
        self.category
    }

    pub fn theta(&self) -> f32 {
        self.theta
    }

    pub fn fsp_score(&self) -> f32 {
        self.fsp
    }

    pub fn has_fsp(&self) -> bool {
        self.has_fsp
    }

    /// No stated formats means any format is fine.
    pub fn accepts_format(&self, format: WorkFormat) -> bool {
        self.formats.is_empty() || self.formats.has(format)
    }

    pub fn text_relevance(&self) -> f32 {
        self.text_relevance
    }

    pub fn text_closeness(&self) -> TextCloseness {
        TextCloseness {
            value: self.text_relevance,
            by_meaning: self.text_by_meaning,
        }
    }

    pub fn salary_expectation(&self) -> Option<u32> {
        (self.salary_min > 0).then_some(self.salary_min)
    }
}

#[derive(Debug, Clone)]
struct CandidateCold {
    id: String,
    fsp_summary: String,
}

/// Structure-of-arrays candidate store: build once (the API caches one per specialization)
/// and rank any number of needs against it without further allocation per candidate.
/// Skills and cold records are shared, so a [`subset`](Self::subset) copies hot records only.
#[derive(Debug, Clone, Default)]
pub struct CandidatePool {
    profiles: Vec<CandidateProfile>,
    skills: Arc<Vec<SkillHit>>,
    cold: Arc<Vec<CandidateCold>>,
}

impl CandidatePool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(candidates: usize) -> Self {
        CandidatePool {
            profiles: Vec::with_capacity(candidates),
            skills: Arc::new(Vec::with_capacity(candidates * 12)),
            cold: Arc::new(Vec::with_capacity(candidates)),
        }
    }

    pub fn from_snapshots(snapshots: &[CandidateSnapshot]) -> Self {
        let mut pool = Self::with_capacity(snapshots.len());
        for s in snapshots {
            pool.push(s);
        }
        pool
    }

    /// Interns the snapshot; duplicate skills keep their most trusted source.
    /// Returns the candidate's index in the pool.
    pub fn push(&mut self, s: &CandidateSnapshot) -> usize {
        let skills = Arc::make_mut(&mut self.skills);
        let start = skills.len();
        for cs in &s.skills {
            if skills.len() - start >= usize::from(u16::MAX) {
                break;
            }
            let id = SkillId::intern(&cs.slug);
            match skills[start..].iter_mut().find(|h| h.id == id) {
                Some(h) if cs.source.precedence() > h.source.precedence() => h.source = cs.source,
                Some(_) => {}
                None => skills.push(SkillHit {
                    id,
                    source: cs.source,
                }),
            }
        }
        let skills_len = (skills.len() - start) as u16;
        let cold = Arc::make_mut(&mut self.cold);
        let salary_min = s
            .salary_expectation_min
            .filter(|v| *v > 0)
            .map_or(0, |v| u32::try_from(v).unwrap_or(u32::MAX));
        self.profiles.push(CandidateProfile {
            last_activity_unix: s.last_activity_unix,
            theta: s.theta as f32,
            fsp: s.fsp.score.clamp(0.0, 1.0),
            text_relevance: (s.text_relevance as f32).clamp(0.0, 1.0),
            salary_min,
            city: s.city.as_deref().map_or(0, lexicon::city_key),
            skills_start: start as u32,
            origin: cold.len() as u32,
            skills_len,
            industries: Bits::of(s.industries.iter().copied()),
            formats: Bits::of(s.work_formats.iter().copied()),
            category: s.category,
            relocation: s.relocation,
            has_fsp: s.fsp.has_history,
            text_by_meaning: s.text_by_meaning,
        });
        cold.push(CandidateCold {
            id: s.id.clone(),
            fsp_summary: s.fsp.summary_ru.clone(),
        });
        self.profiles.len() - 1
    }

    /// A pool of the picked candidates, `(index in self, text closeness)` each, in pick order.
    /// Hot records are copied with the given text closeness; skills and cold records are
    /// shared, so a filtered per-request view of a cached pool costs one allocation.
    pub fn subset(&self, picks: impl IntoIterator<Item = (usize, TextCloseness)>) -> CandidatePool {
        let profiles = picks
            .into_iter()
            .filter_map(|(i, text)| {
                self.profiles.get(i).map(|p| CandidateProfile {
                    text_relevance: text.value.clamp(0.0, 1.0),
                    text_by_meaning: text.by_meaning,
                    ..*p
                })
            })
            .collect();
        CandidatePool {
            profiles,
            skills: Arc::clone(&self.skills),
            cold: Arc::clone(&self.cold),
        }
    }

    pub fn has_skill(&self, index: usize, skill: SkillId) -> bool {
        self.profiles
            .get(index)
            .is_some_and(|p| self.skills_of(p).iter().any(|h| h.id == skill))
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn id(&self, index: usize) -> Option<&str> {
        self.cold_of(index).map(|c| c.id.as_str())
    }

    fn cold_of(&self, index: usize) -> Option<&CandidateCold> {
        self.profiles
            .get(index)
            .and_then(|p| self.cold.get(p.origin as usize))
    }

    pub fn profile(&self, index: usize) -> Option<&CandidateProfile> {
        self.profiles.get(index)
    }

    pub(crate) fn profiles(&self) -> &[CandidateProfile] {
        &self.profiles
    }

    pub(crate) fn skills_of(&self, p: &CandidateProfile) -> &[SkillHit] {
        let start = p.skills_start as usize;
        self.skills
            .get(start..start + usize::from(p.skills_len))
            .unwrap_or(&[])
    }

    /// Per-need text closeness computed outside the database (offline evaluation).
    pub fn set_text_closeness(&mut self, index: usize, text: TextCloseness) {
        if let Some(p) = self.profiles.get_mut(index) {
            p.text_relevance = text.value.clamp(0.0, 1.0);
            p.text_by_meaning = text.by_meaning;
        }
    }

    pub(crate) fn fsp_summary(&self, index: usize) -> &str {
        self.cold_of(index)
            .map_or(NO_FSP_DATA_RU, |c| c.fsp_summary.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::Specialization;

    #[test]
    fn hot_record_stays_small() {
        assert!(std::mem::size_of::<CandidateProfile>() <= 48);
        assert_eq!(std::mem::size_of::<SkillHit>(), 4);
    }

    #[test]
    fn push_dedupes_skills_by_trust() {
        let s = CandidateSnapshot {
            id: "c1".into(),
            category: Category::new(Specialization::Backend, Grade::Middle),
            theta: 0.4,
            skills: vec![
                CandidateSkill {
                    slug: "postgres".into(),
                    source: SkillSource::Declared,
                },
                CandidateSkill {
                    slug: "postgresql".into(),
                    source: SkillSource::Test,
                },
                CandidateSkill {
                    slug: "Haskell".into(),
                    source: SkillSource::Resume,
                },
            ],
            fsp: FspSummary::default(),
            salary_expectation_min: Some(-5),
            work_formats: vec![WorkFormat::Remote],
            city: Some("СПб".into()),
            relocation: false,
            last_activity_unix: 0,
            industries: vec![Industry::Fintech],
            text_relevance: 3.0,
            text_by_meaning: false,
        };
        let pool = CandidatePool::from_snapshots(&[s]);
        let p = pool.profile(0).unwrap();
        let skills = pool.skills_of(p);
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].source, SkillSource::Test);
        assert_eq!(p.salary_min, 0);
        assert_eq!(p.text_relevance, 1.0);
        assert!(p.formats.has(WorkFormat::Remote) && !p.formats.has(WorkFormat::Office));
        assert_eq!(p.city, lexicon::city_key("Санкт-Петербург"));
        assert_eq!(pool.id(0), Some("c1"));
    }

    fn snapshot(id: &str, skill: &str, fsp: &str) -> CandidateSnapshot {
        CandidateSnapshot {
            id: id.into(),
            category: Category::new(Specialization::Backend, Grade::Middle),
            theta: 0.4,
            skills: vec![CandidateSkill {
                slug: skill.into(),
                source: SkillSource::Test,
            }],
            fsp: FspSummary {
                score: 0.5,
                has_history: true,
                summary_ru: fsp.into(),
            },
            salary_expectation_min: Some(250_000),
            work_formats: Vec::new(),
            city: None,
            relocation: false,
            last_activity_unix: 0,
            industries: Vec::new(),
            text_relevance: 0.0,
            text_by_meaning: false,
        }
    }

    #[test]
    fn subset_shares_skills_and_cold_records() {
        let pool = CandidatePool::from_snapshots(&[
            snapshot("a", "rust", "ФСП a"),
            snapshot("b", "go", "ФСП b"),
            snapshot("c", "kafka", "ФСП c"),
        ]);
        let view = pool.subset([
            (2, TextCloseness::keywords(0.25)),
            (0, TextCloseness::meaning(7.0)),
        ]);
        assert_eq!(view.len(), 2);
        assert_eq!(view.id(0), Some("c"));
        assert_eq!(view.id(1), Some("a"));
        assert_eq!(view.fsp_summary(0), "ФСП c");
        assert!(view.has_skill(0, SkillId::intern("kafka")));
        assert!(!view.has_skill(0, SkillId::intern("rust")));
        assert!(view.has_skill(1, SkillId::intern("rust")));
        assert_eq!(
            view.profile(0).unwrap().text_closeness(),
            TextCloseness::keywords(0.25)
        );
        assert_eq!(
            view.profile(1).unwrap().text_closeness(),
            TextCloseness::meaning(1.0)
        );
        assert_eq!(view.profile(1).unwrap().salary_expectation(), Some(250_000));
        assert!(Arc::ptr_eq(&view.skills, &pool.skills));

        let nested = view.subset([(1, TextCloseness::default())]);
        assert_eq!(nested.id(0), Some("a"));
        assert_eq!(nested.id(1), None);
        assert_eq!(pool.profile(0).unwrap().text_relevance(), 0.0);
    }
}
