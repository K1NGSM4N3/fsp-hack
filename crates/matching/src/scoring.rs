//! Hard filters, weighted feature scoring with explanations, and ranking (architecture section 6.3).
//!
//! The ranking loop works on [`CandidatePool`] hot records only and allocates nothing per
//! candidate; texts are produced by [`explain`] for the final top-N.

use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use serde::{Deserialize, Serialize};

use crate::lexicon;
use crate::profile::{
    Bits, CandidatePool, CandidateProfile, CandidateSnapshot, SkillHit, SkillSource,
};
use crate::ru;
use crate::semantic::CLOSE_BY_MEANING;
use crate::skills::SkillId;

/// An employer's hiring need as stored and sent over the API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Need {
    pub specialization: Specialization,
    /// Empty = any grade.
    #[serde(default)]
    pub grades: Vec<Grade>,
    #[serde(default)]
    pub allow_adjacent_grades: bool,
    /// Slugs (aliases and unknown names are accepted).
    #[serde(default)]
    pub must_skills: Vec<String>,
    #[serde(default)]
    pub nice_skills: Vec<String>,
    /// Rubles per month; 0 = not specified.
    #[serde(default)]
    pub salary_min: i64,
    #[serde(default)]
    pub salary_max: i64,
    /// Empty = any format.
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub industries: Vec<Industry>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum FeatureKey {
    Must,
    Test,
    Grade,
    Fsp,
    Nice,
    Salary,
    Fresh,
    Text,
}

const FEATURES: usize = 8;

impl FeatureKey {
    pub const ALL: [FeatureKey; FEATURES] = [
        FeatureKey::Must,
        FeatureKey::Test,
        FeatureKey::Grade,
        FeatureKey::Fsp,
        FeatureKey::Nice,
        FeatureKey::Salary,
        FeatureKey::Fresh,
        FeatureKey::Text,
    ];

    pub const fn label_ru(self) -> &'static str {
        match self {
            FeatureKey::Must => "Обязательные навыки",
            FeatureKey::Test => "Результат теста",
            FeatureKey::Grade => "Грейд",
            FeatureKey::Fsp => "Достижения ФСП",
            FeatureKey::Nice => "Желательные навыки",
            FeatureKey::Salary => "Зарплатные ожидания",
            FeatureKey::Fresh => "Актуальность профиля",
            FeatureKey::Text => "Близость описаний",
        }
    }
}

/// Feature weights; renormalized over the features that apply to a need, so a need
/// without nice-to-have skills still scores on 0..100.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(default)]
pub struct Weights {
    pub must: f32,
    pub test: f32,
    pub grade: f32,
    pub fsp: f32,
    pub nice: f32,
    pub salary: f32,
    pub fresh: f32,
    pub text: f32,
    /// Partial credit through the skill similarity graph; off for the ablation in section 8.2.
    pub use_skill_graph: bool,
}

impl Default for Weights {
    /// Tuned by `crates/validate` (docs/validation/matching.md): the category from the test is
    /// already a hard filter and the grade feature, so θ inside the band only fine-tunes, while
    /// real stack coverage separates relevant candidates best. Stable across seeds.
    fn default() -> Self {
        Weights {
            must: 0.41,
            test: 0.10,
            grade: 0.15,
            fsp: 0.10,
            nice: 0.08,
            salary: 0.07,
            fresh: 0.05,
            text: 0.04,
            use_skill_graph: true,
        }
    }
}

impl Weights {
    pub fn get(&self, key: FeatureKey) -> f32 {
        self.array()[key as usize]
    }

    /// Same weights with one feature switched off (ablations).
    pub fn without(mut self, key: FeatureKey) -> Self {
        *match key {
            FeatureKey::Must => &mut self.must,
            FeatureKey::Test => &mut self.test,
            FeatureKey::Grade => &mut self.grade,
            FeatureKey::Fsp => &mut self.fsp,
            FeatureKey::Nice => &mut self.nice,
            FeatureKey::Salary => &mut self.salary,
            FeatureKey::Fresh => &mut self.fresh,
            FeatureKey::Text => &mut self.text,
        } = 0.0;
        self
    }

    fn array(&self) -> [f32; FEATURES] {
        [
            self.must,
            self.test,
            self.grade,
            self.fsp,
            self.nice,
            self.salary,
            self.fresh,
            self.text,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum FilterReason {
    #[error("Другая специализация")]
    Specialization,
    #[error("Грейд вне запрошенных")]
    Grade,
    #[error("Несовместимый формат работы")]
    WorkFormat,
    #[error("Другой город без готовности к переезду")]
    City,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeatureScore {
    pub key: FeatureKey,
    pub label_ru: String,
    /// 0..1
    pub value: f32,
    /// Effective weight after renormalization; 0 when the feature does not apply.
    pub weight: f32,
    /// Points out of 100: `100 × weight × value`; contributions sum to the total.
    pub contribution: f32,
    pub explanation_ru: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SkillMatch {
    /// Display name of the requested skill.
    pub skill: String,
    pub slug: Option<String>,
    pub required: bool,
    /// Candidate skill that earned the credit (the same skill or a similar one).
    pub matched_by: Option<String>,
    pub source: Option<SkillSource>,
    pub similarity: f32,
    /// `similarity × confidence(source)`
    pub credit: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScoreBreakdown {
    /// 0..100
    pub total: f32,
    pub features: Vec<FeatureScore>,
    pub skills: Vec<SkillMatch>,
    /// Strongest positive reasons, at most four.
    pub reasons: Vec<String>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RankedCandidate {
    /// Index in the [`CandidatePool`] / input slice.
    pub index: usize,
    pub total: f32,
    pub strength: f32,
    /// Present for the first `explain_top` results.
    pub breakdown: Option<ScoreBreakdown>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StrengthRank {
    pub index: usize,
    pub strength: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CategoryStats {
    pub category: Category,
    pub candidates: u32,
    pub with_fsp: u32,
    pub median_salary: Option<i64>,
    /// Mean must-skill coverage of the category's candidates, 0..1.
    pub stack_match: f32,
    /// Grade is among the requested ones.
    pub requested: bool,
}

/// θ above 2.5 is roughly the top 0.6 % of N(0, 1): the practical end of the senior range.
const SENIOR_THETA_CEIL: f32 = 2.5;
/// Unknown expectations neither help nor hurt much: between a fit (1.0) and a clear miss.
const NEUTRAL_SALARY: f32 = 0.7;
/// Expectations 30 % over the budget's upper bound are treated as incompatible.
const SALARY_TOLERANCE: f32 = 0.3;
/// Activity two months ago keeps ~37 % of the freshness score.
const FRESH_DAYS: f32 = 60.0;
const SECONDS_PER_DAY: i64 = 86_400;

/// Position of θ inside the grade's band `[cut, next cut)`, 0..1 — a percentile proxy
/// that is comparable across candidates who took different adaptive tests.
pub fn test_strength(grade: Grade, theta: f32) -> f32 {
    let lo = grade.theta_cut() as f32;
    let hi = grade
        .next()
        .map_or(SENIOR_THETA_CEIL, |g| g.theta_cut() as f32);
    ((theta - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// Inverse of [`test_strength`]: θ at a 0..1 position inside the grade's band.
pub fn theta_for_strength(grade: Grade, strength: f32) -> f32 {
    let lo = grade.theta_cut() as f32;
    let hi = grade
        .next()
        .map_or(SENIOR_THETA_CEIL, |g| g.theta_cut() as f32);
    lo + (hi - lo) * strength.clamp(0.0, 1.0)
}

fn freshness(last_activity_unix: i64, now_unix: i64) -> f32 {
    if last_activity_unix <= 0 {
        return 0.0;
    }
    let days = (now_unix - last_activity_unix).max(0) as f32 / SECONDS_PER_DAY as f32;
    (-days / FRESH_DAYS).exp()
}

/// Ranking inside a category without a need (T-7, M-2): only verified signals plus freshness.
pub fn profile_strength(c: &CandidateProfile, now_unix: i64) -> f32 {
    0.6 * test_strength(c.category.grade, c.theta)
        + 0.25 * c.fsp
        + 0.15 * freshness(c.last_activity_unix, now_unix)
}

/// One grade above is a safer bet than one below: an overqualified candidate can do the job.
fn grade_fit(requested: Bits, g: Grade) -> Option<f32> {
    if requested.is_empty() {
        return None;
    }
    Grade::ALL
        .iter()
        .filter(|r| requested.has(**r))
        .map(|r| match g.steps_from(*r) {
            0 => 1.0,
            1 => 0.6,
            -1 => 0.5,
            _ => 0.0,
        })
        .reduce(f32::max)
}

fn salary_fit(budget_max: u32, expectation: u32) -> f32 {
    if budget_max == 0 || expectation == 0 {
        return NEUTRAL_SALARY;
    }
    if expectation <= budget_max {
        return 1.0;
    }
    let over = (expectation - budget_max) as f32 / budget_max as f32;
    (1.0 - over / SALARY_TOLERANCE).max(0.0)
}

/// Best `similarity × confidence` over the candidate's skills: (credit, similarity, index).
fn best_credit(req: SkillId, skills: &[SkillHit], graph: bool) -> (f32, f32, Option<usize>) {
    let row = if graph { req.sim_row() } else { None };
    let mut best = (0.0f32, 0.0f32, None);
    for (k, h) in skills.iter().enumerate() {
        let sim = if h.id == req {
            1.0
        } else {
            match row {
                Some(row) if h.id.is_known() => row.get(h.id.index()).copied().unwrap_or(0.0),
                _ => 0.0,
            }
        };
        if sim <= 0.0 {
            continue;
        }
        let credit = sim * h.source.confidence();
        if credit > best.0 || (credit == best.0 && sim > best.1) {
            best = (credit, sim, Some(k));
        }
    }
    best
}

fn coverage(req: &[(SkillId, u16)], skills: &[SkillHit], graph: bool) -> f32 {
    if req.is_empty() {
        return 0.0;
    }
    req.iter()
        .map(|&(r, _)| best_credit(r, skills, graph).0)
        .sum::<f32>()
        / req.len() as f32
}

struct Card {
    values: [f32; FEATURES],
    applicable: [bool; FEATURES],
}

impl Card {
    fn total(&self, w: &[f32; FEATURES]) -> f32 {
        let (mut num, mut den) = (0.0f32, 0.0f32);
        for ((&wk, &v), &on) in w.iter().zip(&self.values).zip(&self.applicable) {
            if on {
                num += wk * v;
                den += wk;
            }
        }
        if den > 0.0 {
            (100.0 * num / den).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    fn weight(&self, w: &[f32; FEATURES], k: usize) -> f32 {
        let den: f32 = (0..FEATURES)
            .filter(|&i| self.applicable[i])
            .map(|i| w[i])
            .sum();
        if self.applicable[k] && den > 0.0 {
            w[k] / den
        } else {
            0.0
        }
    }
}

/// [`Need`] with skills interned and sets packed, built once per ranking call.
struct CompiledNeed<'a> {
    need: &'a Need,
    must: Vec<(SkillId, u16)>,
    nice: Vec<(SkillId, u16)>,
    grades: Bits,
    formats: Bits,
    city: u32,
    industries: Bits,
    salary_min: u32,
    salary_max: u32,
    has_text: bool,
}

fn intern_list(list: &[String], exclude: &[(SkillId, u16)]) -> Vec<(SkillId, u16)> {
    let mut out: Vec<(SkillId, u16)> = Vec::with_capacity(list.len());
    for (i, s) in list.iter().enumerate().take(usize::from(u16::MAX)) {
        if s.trim().is_empty() {
            continue;
        }
        let id = SkillId::intern(s);
        if !out.iter().chain(exclude).any(|(x, _)| *x == id) {
            out.push((id, i as u16));
        }
    }
    out
}

fn rubles(v: i64) -> u32 {
    u32::try_from(v.max(0)).unwrap_or(u32::MAX)
}

impl<'a> CompiledNeed<'a> {
    fn new(need: &'a Need) -> Self {
        let must = intern_list(&need.must_skills, &[]);
        let nice = intern_list(&need.nice_skills, &must);
        CompiledNeed {
            need,
            must,
            nice,
            grades: Bits::of(need.grades.iter().copied()),
            formats: Bits::of(need.work_formats.iter().copied()),
            city: need.city.as_deref().map_or(0, lexicon::city_key),
            industries: Bits::of(need.industries.iter().copied()),
            salary_min: rubles(need.salary_min),
            salary_max: rubles(need.salary_max),
            has_text: !need.description.trim().is_empty(),
        }
    }

    fn common_formats(&self, c: &CandidateProfile) -> Bits {
        let all = Bits::all_formats();
        self.formats
            .or_if_empty(all)
            .and(c.formats.or_if_empty(all))
    }

    fn filter(&self, c: &CandidateProfile) -> Result<(), FilterReason> {
        if c.category.specialization != self.need.specialization {
            return Err(FilterReason::Specialization);
        }
        let g = c.category.grade;
        if !self.grades.is_empty() && !self.grades.has(g) {
            let adjacent = self.need.allow_adjacent_grades
                && [g.prev(), g.next()]
                    .into_iter()
                    .flatten()
                    .any(|n| self.grades.has(n));
            if !adjacent {
                return Err(FilterReason::Grade);
            }
        }
        let common = self.common_formats(c);
        if common.is_empty() {
            return Err(FilterReason::WorkFormat);
        }
        // Unknown candidate city passes: it is reported as a gap instead of hiding the person.
        if !common.has(WorkFormat::Remote)
            && self.city != 0
            && c.city != 0
            && c.city != self.city
            && !c.relocation
        {
            return Err(FilterReason::City);
        }
        Ok(())
    }

    fn card(&self, c: &CandidateProfile, skills: &[SkillHit], now_unix: i64, graph: bool) -> Card {
        let mut values = [0.0f32; FEATURES];
        let mut applicable = [true; FEATURES];
        let mut set = |k: FeatureKey, v: Option<f32>| match v {
            Some(v) => values[k as usize] = v,
            None => applicable[k as usize] = false,
        };
        set(
            FeatureKey::Must,
            (!self.must.is_empty()).then(|| coverage(&self.must, skills, graph)),
        );
        set(
            FeatureKey::Test,
            Some(test_strength(c.category.grade, c.theta)),
        );
        set(FeatureKey::Grade, grade_fit(self.grades, c.category.grade));
        set(FeatureKey::Fsp, Some(c.fsp));
        set(
            FeatureKey::Nice,
            (!self.nice.is_empty()).then(|| coverage(&self.nice, skills, graph)),
        );
        set(
            FeatureKey::Salary,
            Some(salary_fit(self.salary_max, c.salary_min)),
        );
        set(
            FeatureKey::Fresh,
            Some(freshness(c.last_activity_unix, now_unix)),
        );
        // Industry is a soft signal (section 4), so it shares the text slot instead of adding a weight.
        let overlap = if c.industries.and(self.industries).is_empty() {
            0.0
        } else {
            1.0
        };
        let text = match (self.has_text, !self.industries.is_empty()) {
            (true, true) => Some(0.6 * c.text_relevance + 0.4 * overlap),
            (true, false) => Some(c.text_relevance),
            (false, true) => Some(overlap),
            (false, false) => None,
        };
        set(FeatureKey::Text, text);
        Card { values, applicable }
    }

    fn requires_presence(&self) -> bool {
        self.city != 0 && !self.formats.is_empty() && !self.formats.has(WorkFormat::Remote)
    }

    fn label(&self, list: &[String], id: SkillId, src: u16) -> String {
        match id.def() {
            Some(d) => d.name.to_owned(),
            None => list
                .get(usize::from(src))
                .map_or_else(String::new, |s| s.trim().to_owned()),
        }
    }
}

pub fn hard_filter(need: &Need, candidate: &CandidateSnapshot) -> Result<(), FilterReason> {
    let mut pool = CandidatePool::with_capacity(1);
    let i = pool.push(candidate);
    match pool.profile(i) {
        Some(p) => CompiledNeed::new(need).filter(p),
        None => Err(FilterReason::Specialization),
    }
}

/// Scores and explains one candidate (no hard filter applied).
pub fn score(
    need: &Need,
    candidate: &CandidateSnapshot,
    weights: &Weights,
    now_unix: i64,
) -> ScoreBreakdown {
    let mut pool = CandidatePool::with_capacity(1);
    let i = pool.push(candidate);
    explain(&CompiledNeed::new(need), &pool, i, weights, now_unix)
}

/// Explains one pool entry, e.g. a single profile opened from search results.
pub fn explain_at(
    need: &Need,
    pool: &CandidatePool,
    index: usize,
    weights: &Weights,
    now_unix: i64,
) -> ScoreBreakdown {
    explain(&CompiledNeed::new(need), pool, index, weights, now_unix)
}

/// Filters and ranks the pool by match score (ties: profile strength, then pool order);
/// builds explanations only for the first `explain_top` results.
pub fn rank(
    need: &Need,
    pool: &CandidatePool,
    weights: &Weights,
    now_unix: i64,
    explain_top: usize,
) -> Vec<RankedCandidate> {
    let cn = CompiledNeed::new(need);
    let w = weights.array();
    let mut scored: Vec<(f32, f32, u32)> = Vec::with_capacity(pool.len());
    for (i, p) in pool.profiles().iter().enumerate() {
        if cn.filter(p).is_err() {
            continue;
        }
        let card = cn.card(p, pool.skills_of(p), now_unix, weights.use_skill_graph);
        scored.push((card.total(&w), profile_strength(p, now_unix), i as u32));
    }
    scored.sort_unstable_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then(b.1.total_cmp(&a.1))
            .then(a.2.cmp(&b.2))
    });
    scored
        .into_iter()
        .enumerate()
        .map(|(pos, (total, strength, i))| RankedCandidate {
            index: i as usize,
            total,
            strength,
            breakdown: (pos < explain_top)
                .then(|| explain(&cn, pool, i as usize, weights, now_unix)),
        })
        .collect()
}

/// Convenience over [`rank`] for callers holding snapshots.
pub fn rank_snapshots(
    need: &Need,
    candidates: &[CandidateSnapshot],
    weights: &Weights,
    now_unix: i64,
    explain_top: usize,
) -> Vec<RankedCandidate> {
    rank(
        need,
        &CandidatePool::from_snapshots(candidates),
        weights,
        now_unix,
        explain_top,
    )
}

/// Category browsing without a need: strongest verified profiles first.
pub fn rank_by_strength(
    pool: &CandidatePool,
    category: Option<Category>,
    now_unix: i64,
) -> Vec<StrengthRank> {
    let mut out: Vec<StrengthRank> = pool
        .profiles()
        .iter()
        .enumerate()
        .filter(|(_, p)| category.is_none_or(|c| p.category == c))
        .map(|(index, p)| StrengthRank {
            index,
            strength: profile_strength(p, now_unix),
        })
        .collect();
    out.sort_unstable_by(|a, b| {
        b.strength
            .total_cmp(&a.strength)
            .then(a.index.cmp(&b.index))
    });
    out
}

/// Recommended-category cards for a need: every grade of the need's specialization with
/// the candidates that pass format/city filters, regardless of the requested grades.
pub fn category_stats(need: &Need, pool: &CandidatePool) -> Vec<CategoryStats> {
    let mut cn = CompiledNeed::new(need);
    let requested = cn.grades;
    cn.grades = Bits::default();
    let mut count = [0u32; 3];
    let mut with_fsp = [0u32; 3];
    let mut coverage_sum = [0.0f32; 3];
    let mut salaries: [Vec<u32>; 3] = Default::default();
    for p in pool.profiles() {
        if cn.filter(p).is_err() {
            continue;
        }
        let g = p.category.grade as usize;
        count[g] += 1;
        with_fsp[g] += u32::from(p.has_fsp);
        coverage_sum[g] += coverage(&cn.must, pool.skills_of(p), true);
        if p.salary_min > 0 {
            salaries[g].push(p.salary_min);
        }
    }
    Grade::ALL
        .iter()
        .map(|&grade| {
            let g = grade as usize;
            salaries[g].sort_unstable();
            let median = match salaries[g].len() {
                0 => None,
                n if n % 2 == 1 => Some(i64::from(salaries[g][n / 2])),
                n => Some((i64::from(salaries[g][n / 2 - 1]) + i64::from(salaries[g][n / 2])) / 2),
            };
            CategoryStats {
                category: Category::new(need.specialization, grade),
                candidates: count[g],
                with_fsp: with_fsp[g],
                median_salary: median,
                stack_match: if count[g] > 0 {
                    coverage_sum[g] / count[g] as f32
                } else {
                    0.0
                },
                requested: requested.is_empty() || requested.has(grade),
            }
        })
        .collect()
}

struct SkillReport {
    matches: Vec<SkillMatch>,
    full: Vec<String>,
    partial: usize,
    missing: Vec<String>,
    verified: usize,
    /// Fully matched by a skill the FSP registry confirms.
    by_fsp: Vec<String>,
}

fn skill_report(
    cn: &CompiledNeed<'_>,
    list: &[String],
    req: &[(SkillId, u16)],
    skills: &[SkillHit],
    graph: bool,
    required: bool,
) -> SkillReport {
    let mut r = SkillReport {
        matches: Vec::with_capacity(req.len()),
        full: Vec::new(),
        partial: 0,
        missing: Vec::new(),
        verified: 0,
        by_fsp: Vec::new(),
    };
    for &(id, src) in req {
        let label = cn.label(list, id, src);
        let (credit, similarity, hit) = best_credit(id, skills, graph);
        let hit = hit.and_then(|k| skills.get(k));
        let matched_by = hit.map(|h| match h.id.def() {
            Some(d) => d.name.to_owned(),
            None => label.clone(),
        });
        if hit.is_none() {
            r.missing.push(label.clone());
        } else if similarity >= 1.0 {
            r.full.push(label.clone());
            if hit.is_some_and(|h| h.source.is_verified()) {
                r.verified += 1;
            }
            if hit.is_some_and(|h| h.source == SkillSource::Fsp) {
                r.by_fsp.push(label.clone());
            }
        } else {
            r.partial += 1;
        }
        r.matches.push(SkillMatch {
            skill: label,
            slug: id.def().map(|d| d.slug.to_owned()),
            required,
            matched_by,
            source: hit.map(|h| h.source),
            similarity,
            credit,
        });
    }
    r
}

const FSP_CONFIRMED: &str = "подтверждено ФСП";
pub const MEANING_REASON: &str = "Опыт близок к задаче по смыслу";
const KEYWORDS_REASON: &str = "Опыт близок к задачам команды";

/// Builds the employer-facing explanation. Reasons are ranked by the points their feature
/// contributed; gaps list what is missing. Absence of FSP data is never a gap (F-3).
fn explain(
    cn: &CompiledNeed<'_>,
    pool: &CandidatePool,
    index: usize,
    weights: &Weights,
    now_unix: i64,
) -> ScoreBreakdown {
    let Some(p) = pool.profile(index) else {
        return ScoreBreakdown {
            total: 0.0,
            features: Vec::new(),
            skills: Vec::new(),
            reasons: Vec::new(),
            gaps: Vec::new(),
        };
    };
    let skills = pool.skills_of(p);
    let graph = weights.use_skill_graph;
    let card = cn.card(p, skills, now_unix, graph);
    let w = weights.array();
    let grade_name = p.category.grade.name_ru();
    let mut reasons: Vec<(f32, String)> = Vec::new();
    let mut gaps: Vec<String> = Vec::new();
    let mut texts: [String; FEATURES] = Default::default();
    let points = |k: FeatureKey| 100.0 * card.weight(&w, k as usize) * card.values[k as usize];

    let must = skill_report(cn, &cn.need.must_skills, &cn.must, skills, graph, true);
    texts[FeatureKey::Must as usize] = if cn.must.is_empty() {
        "Обязательные навыки не заданы".to_owned()
    } else {
        let n = cn.must.len();
        let mut s = format!("Обязательные навыки: {} из {n}", must.full.len());
        if must.partial > 0 {
            s.push_str(&format!(", ещё {} — близкими навыками", must.partial));
        }
        if !must.missing.is_empty() {
            s.push_str(&format!(" (нет {})", must.missing.join(", ")));
        }
        if !must.by_fsp.is_empty() {
            s.push_str(&format!("; {} — {FSP_CONFIRMED}", must.by_fsp.join(", ")));
        }
        let verified = if must.verified > 0 {
            format!(
                "; {} {} тестом или ФСП",
                must.verified,
                ru::plural(
                    must.verified,
                    ["подтверждён", "подтверждены", "подтверждено"]
                )
            )
        } else {
            String::new()
        };
        if must.full.len() == n {
            let head = if n <= 4 {
                format!("Все обязательные навыки: {}", must.full.join(", "))
            } else {
                format!("Все {n} обязательных навыков")
            };
            reasons.push((points(FeatureKey::Must), head + &verified));
        } else if must.full.len() * 2 >= n {
            reasons.push((
                points(FeatureKey::Must),
                format!("{} из {n} обязательных навыков{verified}", must.full.len()),
            ));
        }
        s
    };
    for m in &must.matches {
        match (&m.matched_by, m.source) {
            (None, _) => gaps.push(format!("Нет обязательного навыка: {}", m.skill)),
            (Some(by), _) if m.similarity < 1.0 => {
                gaps.push(format!("Вместо {} — близкий навык {by}", m.skill))
            }
            (Some(_), Some(SkillSource::Declared)) => {
                gaps.push(format!("Навык {} только заявлен, не подтверждён", m.skill));
            }
            _ => {}
        }
    }

    let test = card.values[FeatureKey::Test as usize];
    texts[FeatureKey::Test as usize] = if test >= 0.995 {
        format!("Результат теста у верхней границы грейда {grade_name} и выше")
    } else if test <= 0.005 {
        format!("Результат теста у нижней границы грейда {grade_name}")
    } else {
        format!(
            "Результат теста выше {:.0} % диапазона грейда {grade_name}",
            test * 100.0
        )
    };
    if test >= 0.5 {
        reasons.push((
            points(FeatureKey::Test),
            texts[FeatureKey::Test as usize].clone(),
        ));
    } else if test < 0.2 {
        gaps.push(format!(
            "Результат теста близок к порогу грейда {grade_name}"
        ));
    }

    texts[FeatureKey::Grade as usize] = match grade_fit(cn.grades, p.category.grade) {
        None => "Грейд в запросе не ограничен".to_owned(),
        Some(v) if v >= 1.0 => {
            let text = format!("Грейд {grade_name} — как в запросе");
            // Matching the requested grade is expected after filtering; rank it low.
            reasons.push((0.3 * points(FeatureKey::Grade), text.clone()));
            text
        }
        Some(_) => {
            let above = cn
                .need
                .grades
                .iter()
                .all(|r| p.category.grade.steps_from(*r) > 0);
            let text = if above {
                format!("Грейд {grade_name} — выше запрошенного")
            } else {
                format!("Грейд {grade_name} — ниже запрошенного")
            };
            gaps.push(text.clone());
            text
        }
    };

    texts[FeatureKey::Fsp as usize] = if p.has_fsp {
        let summary = pool.fsp_summary(index).to_owned();
        if p.fsp >= 0.05 {
            reasons.push((points(FeatureKey::Fsp), format!("ФСП: {summary}")));
        }
        summary
    } else {
        crate::fsp::NO_FSP_DATA_RU.to_owned()
    };

    let nice = skill_report(cn, &cn.need.nice_skills, &cn.nice, skills, graph, false);
    texts[FeatureKey::Nice as usize] = if cn.nice.is_empty() {
        "Желательные навыки не заданы".to_owned()
    } else if nice.full.is_empty() {
        format!("Желательные навыки: 0 из {}", cn.nice.len())
    } else {
        reasons.push((
            points(FeatureKey::Nice),
            format!("Желательные навыки: {}", nice.full.join(", ")),
        ));
        let mut s = format!(
            "Желательные навыки: {} из {} — {}",
            nice.full.len(),
            cn.nice.len(),
            nice.full.join(", ")
        );
        if !nice.by_fsp.is_empty() {
            s.push_str(&format!("; {} — {FSP_CONFIRMED}", nice.by_fsp.join(", ")));
        }
        s
    };
    // The share of the skill features these skills earned: each fully matched skill is worth
    // weight / number of skills of its list.
    let share = |k: FeatureKey, list: usize, n: usize| {
        if n == 0 {
            0.0
        } else {
            100.0 * card.weight(&w, k as usize) * list as f32 / n as f32
        }
    };
    let by_fsp: Vec<&str> = must
        .by_fsp
        .iter()
        .chain(&nice.by_fsp)
        .map(String::as_str)
        .collect();
    if !by_fsp.is_empty() {
        reasons.push((
            share(FeatureKey::Must, must.by_fsp.len(), cn.must.len())
                + share(FeatureKey::Nice, nice.by_fsp.len(), cn.nice.len()),
            format!("{} — {FSP_CONFIRMED}", by_fsp.join(", ")),
        ));
    }

    texts[FeatureKey::Salary as usize] = match (cn.salary_max, p.salary_min) {
        (0, _) => "Вилка в запросе не указана".to_owned(),
        (_, 0) => "Ожидания по зарплате не указаны".to_owned(),
        (max, exp) if exp <= max => {
            let text = if cn.salary_min > 0 && exp < cn.salary_min {
                format!("Ожидания {} — ниже вилки", ru::rub(i64::from(exp)))
            } else {
                format!("Ожидания {} — в пределах вилки", ru::rub(i64::from(exp)))
            };
            reasons.push((points(FeatureKey::Salary), text.clone()));
            text
        }
        (max, exp) => {
            let pct = ((exp - max) as f32 / max as f32 * 100.0).round();
            let text = format!(
                "Ожидания {} выше вилки на {pct:.0} %",
                ru::rub(i64::from(exp))
            );
            gaps.push(text.clone());
            text
        }
    };

    texts[FeatureKey::Fresh as usize] = if p.last_activity_unix <= 0 {
        "Нет данных об активности".to_owned()
    } else {
        let days = (now_unix - p.last_activity_unix).max(0) / SECONDS_PER_DAY;
        if days <= 14 {
            reasons.push((
                points(FeatureKey::Fresh),
                format!("Профиль актуален: активность {}", ru::days_ago(days)),
            ));
        } else if days > 90 {
            gaps.push(format!("Давно не было активности: {days} дн."));
        }
        format!("Последняя активность: {}", ru::days_ago(days))
    };

    let shared: Vec<&str> = Industry::ALL
        .iter()
        .filter(|i| p.industries.has(**i) && cn.industries.has(**i))
        .map(|i| i.name_ru())
        .collect();
    let rel = p.text_relevance;
    texts[FeatureKey::Text as usize] = if !card.applicable[FeatureKey::Text as usize] {
        "Описание задач не задано".to_owned()
    } else {
        let mut parts = Vec::new();
        if cn.has_text && p.text_by_meaning {
            parts.push(format!("Близость описаний по смыслу: {:.0} %", rel * 100.0));
            if rel >= CLOSE_BY_MEANING {
                reasons.push((points(FeatureKey::Text), MEANING_REASON.to_owned()));
            }
        } else if cn.has_text {
            parts.push(format!(
                "Близость описаний по ключевым словам: {:.0} %",
                rel * 100.0
            ));
            if rel >= 0.5 {
                reasons.push((points(FeatureKey::Text), KEYWORDS_REASON.to_owned()));
            }
        }
        if !cn.industries.is_empty() {
            if shared.is_empty() {
                parts.push("Нет опыта в отраслях запроса".to_owned());
            } else {
                let text = format!("Опыт в отрасли: {}", shared.join(", "));
                reasons.push((points(FeatureKey::Text), text.clone()));
                parts.push(text);
            }
        }
        parts.join("; ")
    };

    if cn.requires_presence() {
        if p.city == 0 {
            gaps.push("Город не указан".to_owned());
        } else if p.city != cn.city && p.relocation {
            gaps.push("Другой город, есть готовность к переезду".to_owned());
        }
    }

    let features = FeatureKey::ALL
        .iter()
        .zip(texts)
        .map(|(&key, explanation_ru)| {
            let k = key as usize;
            let weight = card.weight(&w, k);
            FeatureScore {
                key,
                label_ru: key.label_ru().to_owned(),
                value: card.values[k],
                weight,
                contribution: 100.0 * weight * card.values[k],
                explanation_ru,
            }
        })
        .collect();
    reasons.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut skill_matches = must.matches;
    skill_matches.extend(nice.matches);
    ScoreBreakdown {
        total: card.total(&w),
        features,
        skills: skill_matches,
        reasons: reasons.into_iter().take(4).map(|(_, r)| r).collect(),
        gaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{CandidateSkill, FspSummary};

    const NOW: i64 = 1_791_158_400;
    const DAY: i64 = 86_400;

    fn need() -> Need {
        Need {
            specialization: Specialization::Backend,
            grades: vec![Grade::Middle],
            allow_adjacent_grades: false,
            must_skills: vec![
                "go".into(),
                "postgresql".into(),
                "kafka".into(),
                "k8s".into(),
            ],
            nice_skills: vec!["redis".into(), "grpc".into()],
            salary_min: 250_000,
            salary_max: 350_000,
            work_formats: vec![WorkFormat::Remote, WorkFormat::Hybrid],
            city: Some("Москва".into()),
            industries: vec![Industry::Fintech],
            description: "Платёжная платформа".into(),
        }
    }

    fn skill(slug: &str, source: SkillSource) -> CandidateSkill {
        CandidateSkill {
            slug: slug.into(),
            source,
        }
    }

    fn candidate(
        id: &str,
        grade: Grade,
        theta: f64,
        skills: Vec<CandidateSkill>,
    ) -> CandidateSnapshot {
        CandidateSnapshot {
            id: id.into(),
            category: Category::new(Specialization::Backend, grade),
            theta,
            skills,
            fsp: FspSummary::default(),
            salary_expectation_min: Some(300_000),
            work_formats: vec![WorkFormat::Remote],
            city: Some("Казань".into()),
            relocation: false,
            last_activity_unix: NOW - 3 * DAY,
            industries: vec![],
            text_relevance: 0.3,
            text_by_meaning: false,
        }
    }

    fn strong() -> CandidateSnapshot {
        candidate(
            "strong",
            Grade::Middle,
            0.82,
            vec![
                skill("go", SkillSource::Test),
                skill("postgresql", SkillSource::Test),
                skill("kafka", SkillSource::Resume),
                skill("kubernetes", SkillSource::Resume),
                skill("redis", SkillSource::Declared),
            ],
        )
    }

    fn weak() -> CandidateSnapshot {
        candidate(
            "weak",
            Grade::Middle,
            0.15,
            vec![
                skill("go", SkillSource::Declared),
                skill("mysql", SkillSource::Resume),
                skill("docker", SkillSource::Resume),
            ],
        )
    }

    #[test]
    fn default_weights_sum_to_one() {
        let w = Weights::default();
        let sum: f32 = FeatureKey::ALL.iter().map(|k| w.get(*k)).sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert_eq!(w.without(FeatureKey::Fsp).fsp, 0.0);
    }

    #[test]
    fn strong_candidate_outranks_weak() {
        let ranked = rank_snapshots(&need(), &[weak(), strong()], &Weights::default(), NOW, 10);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].index, 1);
        assert!(ranked[0].total > ranked[1].total + 20.0, "{ranked:?}");
        let b = ranked[0].breakdown.as_ref().unwrap();
        assert!((b.features.iter().map(|f| f.contribution).sum::<f32>() - b.total).abs() < 1e-3);
        for f in &b.features {
            assert!(!f.explanation_ru.is_empty(), "{:?}", f.key);
        }
        assert!(!b.reasons.is_empty());
        assert!(
            b.reasons.iter().any(|r| r
                .starts_with("Все обязательные навыки: Go, PostgreSQL, Apache Kafka, Kubernetes")),
            "{:?}",
            b.reasons
        );
        let test = b
            .features
            .iter()
            .find(|f| f.key == FeatureKey::Test)
            .unwrap();
        assert_eq!(
            test.explanation_ru,
            "Результат теста выше 82 % диапазона грейда Middle"
        );
    }

    #[test]
    fn fsp_confirmed_skills_are_named_in_the_explanation() {
        let mut n = need();
        n.must_skills = vec!["go".into(), "algorithms".into()];
        n.nice_skills = vec!["data_structures".into()];
        let c = candidate(
            "olympiad",
            Grade::Middle,
            0.5,
            vec![
                skill("go", SkillSource::Declared),
                skill("algorithms", SkillSource::Fsp),
                skill("data_structures", SkillSource::Fsp),
            ],
        );
        let b = score(&n, &c, &Weights::default(), NOW);
        assert!(
            b.reasons
                .contains(&"Алгоритмы, Структуры данных — подтверждено ФСП".to_owned()),
            "{:?}",
            b.reasons
        );
        assert_eq!(
            b.features[0].explanation_ru,
            "Обязательные навыки: 2 из 2; Алгоритмы — подтверждено ФСП"
        );
        let algorithms = b.skills.iter().find(|m| m.skill == "Алгоритмы").unwrap();
        assert_eq!(algorithms.source, Some(SkillSource::Fsp));
        assert!((algorithms.credit - 1.0).abs() < 1e-6);

        // Confirmed by our own test as well: the test is named, the FSP line goes away.
        let both = candidate(
            "both",
            Grade::Middle,
            0.5,
            vec![
                skill("algorithms", SkillSource::Fsp),
                skill("algorithms", SkillSource::Test),
                skill("go", SkillSource::Declared),
            ],
        );
        let b = score(&n, &both, &Weights::default(), NOW);
        let algorithms = b.skills.iter().find(|m| m.skill == "Алгоритмы").unwrap();
        assert_eq!(algorithms.source, Some(SkillSource::Test));
        assert!(!b.reasons.iter().any(|r| r.contains("подтверждено ФСП")));
    }

    #[test]
    fn gaps_name_missing_and_similar_skills() {
        let b = score(&need(), &weak(), &Weights::default(), NOW);
        assert!(
            b.gaps
                .contains(&"Нет обязательного навыка: Apache Kafka".to_owned()),
            "{:?}",
            b.gaps
        );
        assert!(
            b.gaps
                .contains(&"Вместо PostgreSQL — близкий навык MySQL".to_owned())
        );
        assert!(
            b.gaps
                .contains(&"Вместо Kubernetes — близкий навык Docker".to_owned())
        );
        assert!(
            b.gaps
                .contains(&"Навык Go только заявлен, не подтверждён".to_owned())
        );
        let must = &b.features[0];
        assert_eq!(
            must.explanation_ru,
            "Обязательные навыки: 1 из 4, ещё 2 — близкими навыками (нет Apache Kafka)"
        );
        // go 0.5 (declared) + postgresql 0.5×0.7 + kafka 0 + kubernetes 0.5×0.7 = 1.2 / 4
        assert!((must.value - 0.3).abs() < 1e-5);
    }

    #[test]
    fn skill_graph_ablation_removes_partial_credit() {
        let w = Weights {
            use_skill_graph: false,
            ..Weights::default()
        };
        let b = score(&need(), &weak(), &w, NOW);
        assert!((b.features[0].value - 0.125).abs() < 1e-5);
    }

    #[test]
    fn no_fsp_is_neutral_not_a_gap() {
        let b = score(&need(), &strong(), &Weights::default(), NOW);
        let fsp = b
            .features
            .iter()
            .find(|f| f.key == FeatureKey::Fsp)
            .unwrap();
        assert_eq!(fsp.explanation_ru, "Нет данных ФСП");
        assert!(b.gaps.iter().all(|g| !g.contains("ФСП")));

        let mut with_fsp = strong();
        with_fsp.fsp = FspSummary {
            score: 0.6,
            has_history: true,
            summary_ru:
                "Призёр (2 место) всероссийских соревнований по продуктовому программированию, 2025"
                    .into(),
        };
        let ranked = rank_snapshots(&need(), &[strong(), with_fsp], &Weights::default(), NOW, 2);
        assert_eq!(ranked[0].index, 1);
        let reasons = &ranked[0].breakdown.as_ref().unwrap().reasons;
        assert!(
            reasons.iter().any(|r| r.starts_with("ФСП: Призёр")),
            "{reasons:?}"
        );
    }

    #[test]
    fn hard_filters() {
        let n = need();
        assert_eq!(hard_filter(&n, &strong()), Ok(()));

        let mut other_spec = strong();
        other_spec.category.specialization = Specialization::Frontend;
        assert_eq!(
            hard_filter(&n, &other_spec),
            Err(FilterReason::Specialization)
        );

        let senior = candidate("s", Grade::Senior, 1.5, vec![]);
        assert_eq!(hard_filter(&n, &senior), Err(FilterReason::Grade));
        let adjacent = Need {
            allow_adjacent_grades: true,
            ..need()
        };
        assert_eq!(hard_filter(&adjacent, &senior), Ok(()));
        let junior_only = Need {
            grades: vec![Grade::Junior],
            allow_adjacent_grades: true,
            ..need()
        };
        assert_eq!(hard_filter(&junior_only, &senior), Err(FilterReason::Grade));

        let office = Need {
            work_formats: vec![WorkFormat::Office],
            ..need()
        };
        assert_eq!(
            hard_filter(&office, &strong()),
            Err(FilterReason::WorkFormat)
        );

        let mut office_kazan = strong();
        office_kazan.work_formats = vec![WorkFormat::Office, WorkFormat::Hybrid];
        assert_eq!(hard_filter(&office, &office_kazan), Err(FilterReason::City));
        office_kazan.relocation = true;
        assert_eq!(hard_filter(&office, &office_kazan), Ok(()));
        office_kazan.relocation = false;
        office_kazan.city = Some("мск".into());
        assert_eq!(hard_filter(&office, &office_kazan), Ok(()));
        // Hybrid need with remote-capable candidate: remote is not in common formats → city applies.
        let hybrid = Need {
            work_formats: vec![WorkFormat::Hybrid],
            ..need()
        };
        let mut any_format = strong();
        any_format.work_formats.clear();
        assert_eq!(hard_filter(&hybrid, &any_format), Err(FilterReason::City));
        let remote = Need {
            work_formats: vec![WorkFormat::Remote],
            ..need()
        };
        assert_eq!(hard_filter(&remote, &any_format), Ok(()));
    }

    #[test]
    fn salary_and_freshness_curves() {
        assert_eq!(salary_fit(300_000, 300_000), 1.0);
        assert!((salary_fit(300_000, 345_000) - 0.5).abs() < 1e-5);
        assert_eq!(salary_fit(300_000, 400_000), 0.0);
        assert_eq!(salary_fit(300_000, 0), NEUTRAL_SALARY);
        assert!((freshness(NOW - 60 * DAY, NOW) - (-1.0f32).exp()).abs() < 1e-5);
        assert_eq!(freshness(0, NOW), 0.0);
    }

    #[test]
    fn test_strength_uses_grade_band() {
        assert_eq!(test_strength(Grade::Middle, 0.82), 0.82);
        assert_eq!(test_strength(Grade::Junior, -2.0), 0.0);
        assert!((test_strength(Grade::Senior, 1.75) - 0.5).abs() < 1e-6);
        assert_eq!(test_strength(Grade::Senior, 3.0), 1.0);
        for g in Grade::ALL {
            for s in [0.0, 0.25, 0.5, 0.9, 1.0] {
                let back = test_strength(*g, theta_for_strength(*g, s));
                assert!((back - s).abs() < 1e-6, "{g:?} {s}");
            }
        }
        assert_eq!(theta_for_strength(Grade::Middle, 7.0), 1.0);
    }

    #[test]
    fn salary_gap_and_presence_notes() {
        let mut c = strong();
        c.salary_expectation_min = Some(385_000);
        c.work_formats = vec![WorkFormat::Hybrid];
        c.city = None;
        let n = Need {
            work_formats: vec![WorkFormat::Hybrid],
            ..need()
        };
        let b = score(&n, &c, &Weights::default(), NOW);
        assert!(
            b.gaps
                .contains(&"Ожидания 385\u{a0}000\u{a0}₽ выше вилки на 10 %".to_owned()),
            "{:?}",
            b.gaps
        );
        assert!(b.gaps.contains(&"Город не указан".to_owned()));
    }

    #[test]
    fn non_applicable_features_are_renormalized() {
        let bare = Need {
            specialization: Specialization::Backend,
            grades: vec![],
            allow_adjacent_grades: false,
            must_skills: vec![],
            nice_skills: vec![],
            salary_min: 0,
            salary_max: 0,
            work_formats: vec![],
            city: None,
            industries: vec![],
            description: String::new(),
        };
        let b = score(&bare, &strong(), &Weights::default(), NOW);
        let weight_sum: f32 = b.features.iter().map(|f| f.weight).sum();
        assert!((weight_sum - 1.0).abs() < 1e-5);
        assert_eq!(b.features[0].weight, 0.0);
        assert_eq!(
            b.features[0].explanation_ru,
            "Обязательные навыки не заданы"
        );
    }

    #[test]
    fn strength_ranking_prefers_test_and_fsp() {
        let mut fsp = weak();
        fsp.theta = 0.82;
        fsp.fsp.score = 0.8;
        fsp.fsp.has_history = true;
        let pool = CandidatePool::from_snapshots(&[weak(), strong(), fsp]);
        let order: Vec<usize> = rank_by_strength(&pool, None, NOW)
            .iter()
            .map(|r| r.index)
            .collect();
        assert_eq!(order, vec![2, 1, 0]);
    }

    #[test]
    fn category_cards() {
        let senior = candidate(
            "s",
            Grade::Senior,
            1.5,
            vec![skill("go", SkillSource::Test)],
        );
        let pool = CandidatePool::from_snapshots(&[strong(), weak(), senior]);
        let stats = category_stats(&need(), &pool);
        assert_eq!(stats.len(), 3);
        let middle = &stats[1];
        assert!(middle.requested);
        assert_eq!(middle.candidates, 2);
        assert_eq!(middle.median_salary, Some(300_000));
        assert!(!stats[2].requested);
        assert_eq!(stats[2].candidates, 1);
        assert!(middle.stack_match > stats[2].stack_match);
    }

    #[test]
    fn explanations_only_for_top_n() {
        let ranked = rank_snapshots(&need(), &[weak(), strong()], &Weights::default(), NOW, 1);
        assert!(ranked[0].breakdown.is_some());
        assert!(ranked[1].breakdown.is_none());
    }

    #[test]
    fn text_closeness_explains_its_method() {
        let text = |b: &ScoreBreakdown| {
            b.features
                .iter()
                .find(|f| f.key == FeatureKey::Text)
                .unwrap()
                .clone()
        };
        let mut c = strong();
        c.text_relevance = 0.72;
        c.text_by_meaning = true;
        let mut n = need();
        n.industries.clear();
        // Heavy text weight: the reason must make the top four.
        let w = Weights {
            text: 0.6,
            ..Weights::default()
        };
        let b = score(&n, &c, &w, NOW);
        let f = text(&b);
        assert_eq!(f.label_ru, "Близость описаний");
        assert_eq!(f.explanation_ru, "Близость описаний по смыслу: 72 %");
        assert!((f.value - 0.72).abs() < 1e-6);
        assert!(
            b.reasons.iter().any(|r| r == MEANING_REASON),
            "{:?}",
            b.reasons
        );

        c.text_relevance = 0.5;
        let b = score(&n, &c, &w, NOW);
        assert_eq!(text(&b).explanation_ru, "Близость описаний по смыслу: 50 %");
        assert!(
            !b.reasons.iter().any(|r| r == MEANING_REASON),
            "below the threshold"
        );

        c.text_by_meaning = false;
        c.text_relevance = 0.6;
        let b = score(&n, &c, &w, NOW);
        assert_eq!(
            text(&b).explanation_ru,
            "Близость описаний по ключевым словам: 60 %"
        );
        assert!(b.reasons.iter().any(|r| r == KEYWORDS_REASON));

        n.description.clear();
        assert_eq!(
            text(&score(&n, &c, &w, NOW)).explanation_ru,
            "Описание задач не задано"
        );
    }
}
