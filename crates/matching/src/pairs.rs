//! Offline evaluation on external "vacancy – candidate" pairs, e.g. the jury's own set. Every
//! vacancy is matched by the production parsers and scorer against all candidates, a keyword
//! baseline ranks the same candidates, and expert labels, when given, turn both rankings into
//! metrics. Nothing is invented for missing data: a category recognized from a resume is marked
//! as not confirmed by a test, a missing test result is a stated neutral value.

mod input;
mod report;
mod terms;

use std::collections::HashMap;

use catalog::{Category, Grade};

pub use input::{
    CandidateInput, DEFAULT_K, LabelInput, MAX_CANDIDATES, MAX_ID_CHARS, MAX_K, MAX_LABELS,
    MAX_LIST_ITEMS, MAX_SHORT_CHARS, MAX_TEXT_CHARS, MAX_VACANCIES, PairsError, PairsInput,
    VacancyInput,
};
pub use report::{
    BaselinePair, CandidateSummary, Coverage, Mean, MeanMetrics, Metrics, MetricsSummary, Origin,
    OriginCounts, PairsReport, RankedPair, VacancyReport, VacancySummary,
};

use crate::fsp::fsp_score;
use crate::metrics::{self, RELEVANT};
use crate::need_parse::parse_need;
use crate::profile::{CandidatePool, CandidateSkill, CandidateSnapshot, FspSummary, SkillSource};
use crate::resume::parse_resume;
use crate::scoring::{FeatureKey, Need, ScoreBreakdown, Weights, rank, theta_for_strength};
use crate::skills::SkillId;
use crate::text::fnv1a;

/// Test strength for a candidate without a result: the middle of the grade band, where a
/// typical member of the category sits, so the missing test neither helps nor hurts.
pub const NEUTRAL_TEST_STRENGTH: f32 = 0.5;
/// A vacancy specialization guessed with less confidence is reported as unreliable.
pub const LOW_CONFIDENCE: f32 = 0.5;
pub const NO_TEST_RU: &str =
    "Результата теста нет: взято нейтральное значение — середина диапазона грейда";

const UNLABELED: u8 = u8::MAX;
const TITLE_CHARS: usize = 120;
const SUMMARY_SKILLS: usize = 30;
const DAY: i64 = 86_400;

/// Experience filter of the keyword baseline (shared with `validate`): what a job board does
/// with "опыт 1–3 года" when the vacancy names a grade.
pub fn years_fit(grade: Grade, years: f64) -> bool {
    match grade {
        Grade::Junior => years < 2.5,
        Grade::Middle => (2.0..6.0).contains(&years),
        Grade::Senior => years >= 5.0,
    }
}

/// Hot per-candidate data for the baseline and the text feature, structure of arrays indexed by
/// candidate; the production [`CandidatePool`] holds only candidates that have a category.
struct Hot {
    pool: CandidatePool,
    pool_to_candidate: Vec<u32>,
    /// Stated experience in years, NaN = unknown.
    years: Vec<f32>,
    keyword_spans: Vec<(u32, u16)>,
    keywords: Vec<SkillId>,
    term_spans: Vec<(u32, u32)>,
    terms: Vec<u32>,
    id_hash: Vec<u32>,
}

impl Hot {
    fn keywords_of(&self, c: usize) -> &[SkillId] {
        let (start, len) = self.keyword_spans[c];
        &self.keywords[start as usize..start as usize + usize::from(len)]
    }

    fn terms_of(&self, c: usize) -> &[u32] {
        let (start, len) = self.term_spans[c];
        &self.terms[start as usize..(start + len) as usize]
    }
}

/// Cold per-candidate facts, read only for notes and the report.
struct Facts {
    category: Option<Category>,
    specialization_origin: Origin,
    specialization_confidence: Option<f32>,
    grade_origin: Origin,
    test_result: Option<f32>,
    experience_years: Option<f32>,
    skills: Vec<String>,
    has_fsp: bool,
}

impl Facts {
    fn notes(&self) -> Vec<String> {
        let mut out = Vec::new();
        let Some(cat) = self.category else {
            return out;
        };
        match (self.specialization_origin, self.grade_origin) {
            (Origin::Text, Origin::Text) => out.push(format!(
                "Категория «{}» — из резюме, не подтверждена тестом",
                cat.name_ru()
            )),
            (spec, grade) => {
                if spec == Origin::Text {
                    out.push(format!(
                        "Специализация «{}» — из резюме, не подтверждена тестом",
                        cat.specialization.name_ru()
                    ));
                }
                match grade {
                    Origin::Text => out.push(format!(
                        "Грейд {} — из резюме, не подтверждён тестом",
                        cat.grade.name_ru()
                    )),
                    Origin::Missing => out.push(format!(
                        "Грейд не найден ни во входных данных, ни в резюме — принят {}",
                        cat.grade.name_ru()
                    )),
                    Origin::Input => {}
                }
            }
        }
        if self.test_result.is_none() {
            out.push(NO_TEST_RU.to_owned());
        }
        out
    }
}

fn read_candidates(input: &[CandidateInput], now: i64) -> (Hot, Vec<Facts>) {
    let n = input.len();
    let mut snapshots: Vec<CandidateSnapshot> = Vec::with_capacity(n);
    let mut hot = Hot {
        pool: CandidatePool::default(),
        pool_to_candidate: Vec::with_capacity(n),
        years: Vec::with_capacity(n),
        keyword_spans: Vec::with_capacity(n),
        keywords: Vec::with_capacity(n * 16),
        term_spans: Vec::with_capacity(n),
        terms: Vec::with_capacity(n * 256),
        id_hash: Vec::with_capacity(n),
    };
    let mut facts = Vec::with_capacity(n);
    for (i, c) in input.iter().enumerate() {
        let parsed = parse_resume(&c.resume_text, now);
        let (spec, specialization_origin, specialization_confidence) =
            match (c.specialization, parsed.specialization_hint) {
                (Some(s), _) => (Some(s), Origin::Input, None),
                (None, Some(g)) => (Some(g.specialization), Origin::Text, Some(g.confidence)),
                (None, None) => (None, Origin::Missing, None),
            };
        let (grade, grade_origin) = match (c.grade, parsed.grade_hint) {
            (Some(g), _) => (g, Origin::Input),
            (None, Some(g)) => (g, Origin::Text),
            (None, None) => (Grade::Middle, Origin::Missing),
        };

        let mut skills: Vec<CandidateSkill> =
            Vec::with_capacity(c.skills.len() + parsed.skills.len());
        skills.extend(
            c.skills
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| CandidateSkill {
                    slug: s.to_owned(),
                    source: SkillSource::Declared,
                }),
        );
        skills.extend(parsed.skills.iter().map(|s| CandidateSkill {
            slug: s.clone(),
            source: SkillSource::Resume,
        }));
        let start = hot.keywords.len();
        let mut summary_skills = Vec::new();
        for s in &skills {
            let id = SkillId::intern(&s.slug);
            if hot.keywords[start..].contains(&id) {
                continue;
            }
            hot.keywords.push(id);
            if summary_skills.len() < SUMMARY_SKILLS {
                summary_skills.push(
                    id.def()
                        .map_or_else(|| s.slug.clone(), |d| d.slug.to_owned()),
                );
            }
        }
        hot.keyword_spans
            .push((start as u32, (hot.keywords.len() - start) as u16));
        let term_start = hot.terms.len();
        let added = terms::document(&c.resume_text, &mut hot.terms);
        hot.term_spans.push((term_start as u32, added as u32));
        hot.years.push(parsed.experience_years.unwrap_or(f32::NAN));
        hot.id_hash.push(fnv1a(&c.id));

        let category = spec.map(|s| Category::new(s, grade));
        if let Some(category) = category {
            let strength = c.test_result.map_or(NEUTRAL_TEST_STRENGTH, |t| t / 100.0);
            hot.pool_to_candidate.push(i as u32);
            snapshots.push(CandidateSnapshot {
                id: c.id.clone(),
                category,
                theta: f64::from(theta_for_strength(grade, strength)),
                skills,
                fsp: FspSummary::from(&fsp_score(
                    &c.fsp_achievements,
                    category.specialization,
                    now,
                )),
                salary_expectation_min: c.salary_expectation.or(parsed.salary_expectation),
                work_formats: if c.work_formats.is_empty() {
                    parsed.work_formats
                } else {
                    c.work_formats.clone()
                },
                city: c.city.clone().or(parsed.city),
                relocation: c.relocation.or(parsed.relocation).unwrap_or(false),
                last_activity_unix: 0,
                industries: parsed.industries,
                text_relevance: 0.0,
            });
        }
        facts.push(Facts {
            category,
            specialization_origin,
            specialization_confidence,
            grade_origin,
            test_result: c.test_result,
            experience_years: parsed.experience_years,
            skills: summary_skills,
            has_fsp: !c.fsp_achievements.is_empty(),
        });
    }
    hot.pool = CandidatePool::from_snapshots(&snapshots);
    (hot, facts)
}

/// A vacancy compiled for matching: the need (when the specialization is known), the baseline's
/// keywords and grades, and the text-closeness query.
struct Plan {
    need: Option<Need>,
    summary: VacancySummary,
    keywords: Vec<SkillId>,
    query: Vec<u32>,
    title: String,
    notes: Vec<String>,
}

fn names(list: &[String]) -> Vec<String> {
    list.iter()
        .map(|s| {
            SkillId::intern(s)
                .def()
                .map_or_else(|| s.trim().to_owned(), |d| d.name.to_owned())
        })
        .collect()
}

fn overrides(list: &[String]) -> Vec<String> {
    list.iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn plan(v: &VacancyInput) -> Plan {
    let parsed = parse_need(&v.text);
    let (specialization, specialization_origin, specialization_confidence) =
        match (v.specialization, parsed.specialization) {
            (Some(s), _) => (Some(s), Origin::Input, None),
            (None, Some(g)) => (Some(g.specialization), Origin::Text, Some(g.confidence)),
            (None, None) => (None, Origin::Missing, None),
        };
    let (grades, grades_origin) = if !v.grades.is_empty() {
        let mut g = v.grades.clone();
        g.sort_unstable();
        g.dedup();
        (g, Origin::Input)
    } else {
        let g = if parsed.grades.is_empty() {
            parsed.grade.into_iter().collect()
        } else {
            parsed.grades
        };
        let origin = if g.is_empty() {
            Origin::Missing
        } else {
            Origin::Text
        };
        (g, origin)
    };
    let (must, nice, skills_origin) = if v.must_skills.is_empty() && v.nice_skills.is_empty() {
        let origin = if parsed.must_skills.is_empty() && parsed.nice_skills.is_empty() {
            Origin::Missing
        } else {
            Origin::Text
        };
        (parsed.must_skills, parsed.nice_skills, origin)
    } else {
        (
            overrides(&v.must_skills),
            overrides(&v.nice_skills),
            Origin::Input,
        )
    };
    let (salary_min, salary_max) = if v.salary_min.is_some() || v.salary_max.is_some() {
        (v.salary_min, v.salary_max)
    } else {
        (parsed.salary_min, parsed.salary_max)
    };
    let work_formats = if v.work_formats.is_empty() {
        parsed.work_formats
    } else {
        v.work_formats.clone()
    };
    let city = v
        .city
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .or(parsed.city);
    let allow_adjacent_grades = v.allow_adjacent_grades.unwrap_or(true);
    let low_confidence = specialization.is_none()
        || specialization_confidence.is_some_and(|c| c < LOW_CONFIDENCE)
        || skills_origin == Origin::Missing;

    let mut keywords: Vec<SkillId> = Vec::with_capacity(must.len() + nice.len());
    for s in must.iter().chain(&nice) {
        let id = SkillId::intern(s);
        if !keywords.contains(&id) {
            keywords.push(id);
        }
    }
    let mut notes = Vec::new();
    if specialization.is_none() {
        notes.push(
            "Специализация не распознана: наш подбор для вакансии пуст, базовая линия считается"
                .to_owned(),
        );
    }
    if grades.is_empty() {
        notes.push("Грейд не указан и не распознан: подбор без фильтра по грейду".to_owned());
    }
    if skills_origin == Origin::Missing {
        notes.push("Навыки не указаны и не распознаны: признаки навыков не участвуют".to_owned());
    }
    let summary = VacancySummary {
        specialization,
        specialization_origin,
        specialization_confidence,
        grades: grades.clone(),
        grades_origin,
        allow_adjacent_grades,
        must_skills: names(&must),
        nice_skills: names(&nice),
        skills_origin,
        salary_min,
        salary_max,
        work_formats: work_formats.clone(),
        city: city.clone(),
        low_confidence,
    };
    let need = specialization.map(|specialization| Need {
        specialization,
        grades,
        allow_adjacent_grades,
        must_skills: must,
        nice_skills: nice,
        salary_min: salary_min.unwrap_or(0),
        salary_max: salary_max.unwrap_or(0),
        work_formats,
        city,
        industries: parsed.industries,
        description: v.text.trim().to_owned(),
    });
    let title: String = v
        .text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or(&v.id)
        .chars()
        .take(TITLE_CHARS)
        .collect();
    Plan {
        need,
        summary,
        keywords,
        query: terms::query(&v.text),
        title,
        notes,
    }
}

/// Production normalizes `ts_rank` by the best candidate of the loaded pool; the same here
/// over the candidates of the need's specialization.
fn set_text_relevance(hot: &mut Hot, need: &Need, query: &[u32], overlap: &mut Vec<u16>) {
    overlap.clear();
    let mut max = 0u16;
    for (i, &c) in hot.pool_to_candidate.iter().enumerate() {
        let same = hot
            .pool
            .profile(i)
            .is_some_and(|p| p.category().specialization == need.specialization);
        let o = if same {
            terms::overlap(query, hot.terms_of(c as usize))
        } else {
            0
        };
        max = max.max(o);
        overlap.push(o);
    }
    for (i, &o) in overlap.iter().enumerate() {
        let v = if max > 0 {
            f32::from(o) / f32::from(max)
        } else {
            0.0
        };
        hot.pool.set_text_relevance(i, v);
    }
}

/// Murmur3 finalizer. Equal keyword counts are ordered pseudo-randomly but reproducibly, as the
/// validation baseline breaks ties with random noise: input order must not leak into the ranking.
fn mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// "Ключевые слова резюме + заявленный стаж" from `validate`: candidates whose stated experience
/// contradicts the vacancy grade are dropped, the rest are ordered by how many of the vacancy's
/// skills appear among their resume keywords. Unknown experience is not filtered.
fn baseline(hot: &Hot, plan: &Plan, salt: u32) -> Vec<(u32, u16)> {
    let grades = &plan.summary.grades;
    let mut scored: Vec<(u16, u32, u32)> = Vec::with_capacity(hot.years.len());
    for (c, &years) in hot.years.iter().enumerate() {
        if !grades.is_empty()
            && !years.is_nan()
            && !grades.iter().any(|g| years_fit(*g, f64::from(years)))
        {
            continue;
        }
        let kw = hot.keywords_of(c);
        let hits = plan.keywords.iter().filter(|k| kw.contains(k)).count() as u16;
        scored.push((hits, mix(hot.id_hash[c] ^ salt), c as u32));
    }
    scored.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    scored.into_iter().map(|(h, _, c)| (c, h)).collect()
}

/// Dense vacancy × candidate relevance grid; [`UNLABELED`] where no label was given.
struct Labels {
    grid: Vec<u8>,
    candidates: usize,
}

impl Labels {
    fn row(&self, v: usize) -> Option<&[u8]> {
        self.grid
            .get(v * self.candidates..(v + 1) * self.candidates)
    }
}

fn index<'a, T>(
    list: &'a [T],
    id: impl Fn(&T) -> &str,
    field: &'static str,
) -> Result<HashMap<&'a str, u32>, PairsError> {
    let mut map: HashMap<&str, u32> = HashMap::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        if map.insert(id(item), i as u32).is_some() {
            return Err(PairsError::Invalid {
                field,
                path: format!("{field}[{i}].id"),
                message: format!("«{}» уже встречался", id(item)),
            });
        }
    }
    Ok(map)
}

fn read_labels(input: &PairsInput) -> Result<Labels, PairsError> {
    let vacancies = index(&input.vacancies, |v| v.id.as_str(), "vacancies")?;
    let candidates = index(&input.candidates, |c| c.id.as_str(), "candidates")?;
    let n = input.candidates.len();
    let grid_len = if input.labels.is_empty() {
        0
    } else {
        input.vacancies.len() * n
    };
    let mut grid = vec![UNLABELED; grid_len];
    for (i, l) in input.labels.iter().enumerate() {
        let invalid = |message: String| PairsError::Invalid {
            field: "labels",
            path: format!("labels[{i}]"),
            message,
        };
        let v = *vacancies
            .get(l.vacancy_id.as_str())
            .ok_or_else(|| invalid(format!("нет вакансии «{}»", l.vacancy_id)))?;
        let c = *candidates
            .get(l.candidate_id.as_str())
            .ok_or_else(|| invalid(format!("нет кандидата «{}»", l.candidate_id)))?;
        let cell = &mut grid[v as usize * n + c as usize];
        if *cell != UNLABELED {
            return Err(invalid(format!(
                "пара «{}» – «{}» уже размечена",
                l.vacancy_id, l.candidate_id
            )));
        }
        *cell = l.relevance;
    }
    Ok(Labels {
        grid,
        candidates: n,
    })
}

fn grade_of(row: Option<&[u8]>, c: u32) -> Option<u8> {
    row.and_then(|r| r.get(c as usize))
        .copied()
        .filter(|&g| g != UNLABELED)
}

fn ranking_metrics(order: &[u32], row: &[u8], all: &[u8], relevant: usize, k: usize) -> Metrics {
    let grades: Vec<u8> = order
        .iter()
        .map(|&c| grade_of(Some(row), c).unwrap_or(0))
        .collect();
    let shown = order.len().min(k);
    let judged = order[..shown]
        .iter()
        .filter(|&&c| grade_of(Some(row), c).is_some())
        .count();
    Metrics {
        precision_at_5: metrics::precision_at(&grades, 5),
        precision_at_k: metrics::precision_at(&grades, k),
        ndcg_at_k: metrics::ndcg_at(&grades, all, k),
        average_precision: metrics::average_precision(&grades, relevant),
        reciprocal_rank: metrics::reciprocal_rank(&grades),
        judged_at_k: if shown == 0 {
            0.0
        } else {
            judged as f64 / shown as f64
        },
    }
}

fn mean_metrics(list: &[Metrics]) -> MeanMetrics {
    let mean = |f: fn(&Metrics) -> f64| {
        let s = metrics::summarize(&list.iter().map(f).collect::<Vec<_>>());
        Mean {
            mean: s.mean,
            ci95: s.ci95,
        }
    };
    MeanMetrics {
        precision_at_5: mean(|m| m.precision_at_5),
        precision_at_k: mean(|m| m.precision_at_k),
        ndcg_at_k: mean(|m| m.ndcg_at_k),
        average_precision: mean(|m| m.average_precision),
        reciprocal_rank: mean(|m| m.reciprocal_rank),
        judged_at_k: mean(|m| m.judged_at_k),
    }
}

/// Explanation of a candidate without a test result must not cite one.
fn without_test_claims(mut b: ScoreBreakdown) -> ScoreBreakdown {
    if let Some(f) = b.features.iter_mut().find(|f| f.key == FeatureKey::Test) {
        let claimed = std::mem::replace(&mut f.explanation_ru, NO_TEST_RU.to_owned());
        b.reasons.retain(|r| *r != claimed);
    }
    b
}

/// `YYYY-MM-DD` → unix seconds at midnight UTC.
pub fn parse_day(s: &str) -> Option<i64> {
    let mut parts = s.trim().splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    let date =
        time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()?;
    Some(date.midnight().assume_utc().unix_timestamp())
}

fn format_day(unix: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(unix).map_or_else(
        |_| String::from("1970-01-01"),
        |t| format!("{:04}-{:02}-{:02}", t.year(), u8::from(t.month()), t.day()),
    )
}

/// Runs the evaluation. `now_unix` is used when the input has no `now`; either is truncated to
/// the day, so the report's date fully determines the result.
pub fn evaluate(input: &PairsInput, now_unix: i64) -> Result<PairsReport, PairsError> {
    input.check()?;
    let now = match &input.now {
        Some(s) => parse_day(s).ok_or_else(|| PairsError::Invalid {
            field: "now",
            path: "now".into(),
            message: "дата в формате ГГГГ-ММ-ДД".into(),
        })?,
        None => now_unix,
    }
    .div_euclid(DAY)
        * DAY;
    let k = input.k.unwrap_or(DEFAULT_K);
    let labels = read_labels(input)?;
    let (mut hot, facts) = read_candidates(&input.candidates, now);
    let plans: Vec<Plan> = input.vacancies.iter().map(plan).collect();
    let weights = Weights::default();

    let mut overlap = Vec::with_capacity(hot.pool.len());
    let mut evaluated_ours = Vec::new();
    let mut evaluated_base = Vec::new();
    let mut vacancies = Vec::with_capacity(plans.len());
    let mut labeled_vacancies = 0;
    for (vi, (v, plan)) in input.vacancies.iter().zip(&plans).enumerate() {
        let ranked = match &plan.need {
            Some(need) => {
                set_text_relevance(&mut hot, need, &plan.query, &mut overlap);
                rank(need, &hot.pool, &weights, now, k)
            }
            None => Vec::new(),
        };
        let ours: Vec<u32> = ranked
            .iter()
            .map(|r| hot.pool_to_candidate[r.index])
            .collect();
        let base = baseline(&hot, plan, fnv1a(&v.id));
        let base_order: Vec<u32> = base.iter().map(|&(c, _)| c).collect();

        let row = labels.row(vi);
        let judged: Vec<u8> = row
            .map(|r| r.iter().copied().filter(|&g| g != UNLABELED).collect())
            .unwrap_or_default();
        let relevant = judged.iter().filter(|&&g| g >= RELEVANT).count();
        labeled_vacancies += usize::from(!judged.is_empty());
        let mut notes = plan.notes.clone();
        let (ours_metrics, base_metrics) = match row {
            Some(row) if relevant > 0 => {
                let o = ranking_metrics(&ours, row, &judged, relevant, k);
                let b = ranking_metrics(&base_order, row, &judged, relevant, k);
                evaluated_ours.push(o);
                evaluated_base.push(b);
                (Some(o), Some(b))
            }
            _ => {
                if !judged.is_empty() {
                    notes.push(
                        "В разметке нет релевантных кандидатов: вакансия не входит в средние"
                            .to_owned(),
                    );
                }
                (None, None)
            }
        };

        let top = ranked
            .into_iter()
            .take(k)
            .enumerate()
            .filter_map(|(pos, r)| {
                let c = hot.pool_to_candidate[r.index];
                let f = &facts[c as usize];
                let breakdown = r.breakdown?;
                Some(RankedPair {
                    rank: pos + 1,
                    candidate_id: input.candidates[c as usize].id.clone(),
                    score: r.total,
                    relevance: grade_of(row, c),
                    category: f.category?,
                    notes: f.notes(),
                    breakdown: if f.test_result.is_some() {
                        breakdown
                    } else {
                        without_test_claims(breakdown)
                    },
                })
            })
            .collect();
        let baseline_top = base
            .iter()
            .take(k)
            .enumerate()
            .map(|(pos, &(c, _))| {
                let kw = hot.keywords_of(c as usize);
                BaselinePair {
                    rank: pos + 1,
                    candidate_id: input.candidates[c as usize].id.clone(),
                    keywords: plan
                        .keywords
                        .iter()
                        .filter(|id| kw.contains(id))
                        .map(|id| id.def().map_or_else(String::new, |d| d.name.to_owned()))
                        .collect(),
                    experience_years: facts[c as usize].experience_years,
                    relevance: grade_of(row, c),
                }
            })
            .collect();
        vacancies.push(VacancyReport {
            id: v.id.clone(),
            title: plan.title.clone(),
            need: plan.summary.clone(),
            passed_filters: ours.len(),
            labels: judged.len(),
            relevant_labels: relevant,
            ours: ours_metrics,
            baseline: base_metrics,
            top,
            baseline_top,
            notes,
        });
    }

    let mut coverage = Coverage {
        vacancies: plans.len(),
        vacancy_specialization: OriginCounts::default(),
        vacancy_grades: OriginCounts::default(),
        vacancy_skills: OriginCounts::default(),
        vacancies_low_confidence: Vec::new(),
        candidates: facts.len(),
        candidate_specialization: OriginCounts::default(),
        candidate_grade: OriginCounts::default(),
        candidates_without_category: Vec::new(),
        candidates_with_test: 0,
        candidates_with_fsp: 0,
        labels: input.labels.len(),
        labeled_vacancies,
    };
    for (v, p) in input.vacancies.iter().zip(&plans) {
        coverage
            .vacancy_specialization
            .add(p.summary.specialization_origin);
        coverage.vacancy_grades.add(p.summary.grades_origin);
        coverage.vacancy_skills.add(p.summary.skills_origin);
        if p.summary.low_confidence {
            coverage.vacancies_low_confidence.push(v.id.clone());
        }
    }
    let candidates = input
        .candidates
        .iter()
        .zip(facts)
        .map(|(c, f)| {
            coverage
                .candidate_specialization
                .add(f.specialization_origin);
            if f.category.is_some() {
                coverage.candidate_grade.add(f.grade_origin);
            } else {
                coverage.candidates_without_category.push(c.id.clone());
            }
            coverage.candidates_with_test += usize::from(f.test_result.is_some());
            coverage.candidates_with_fsp += usize::from(f.has_fsp);
            CandidateSummary {
                id: c.id.clone(),
                category: f.category,
                specialization_origin: f.specialization_origin,
                specialization_confidence: f.specialization_confidence,
                grade_origin: f.grade_origin,
                test_result: f.test_result,
                experience_years: f.experience_years,
                skills: f.skills,
                has_fsp: f.has_fsp,
            }
        })
        .collect();

    Ok(PairsReport {
        now: format_day(now),
        k,
        relevant_from: RELEVANT,
        neutral_test_strength: NEUTRAL_TEST_STRENGTH,
        coverage,
        metrics: (!evaluated_ours.is_empty()).then(|| MetricsSummary {
            vacancies: evaluated_ours.len(),
            ours: mean_metrics(&evaluated_ours),
            baseline: mean_metrics(&evaluated_base),
        }),
        vacancies,
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use catalog::Specialization;

    use super::*;

    const NOW: i64 = 1_790_812_800; // 2026-10-01

    fn vacancy() -> VacancyInput {
        VacancyInput {
            id: "go".into(),
            text: "Go-разработчик (Middle)\nТребования: Go, PostgreSQL, Kafka.\nУдалённо, 250–330 тыс. ₽."
                .into(),
            ..Default::default()
        }
    }

    fn go_resume(id: &str) -> CandidateInput {
        CandidateInput {
            id: id.into(),
            resume_text: "Go-разработчик\nОпыт работы — 4 года\nGo, PostgreSQL, Kafka, Redis."
                .into(),
            ..Default::default()
        }
    }

    fn input(candidates: Vec<CandidateInput>, labels: Vec<(&str, &str, u8)>) -> PairsInput {
        PairsInput {
            now: None,
            k: Some(5),
            vacancies: vec![vacancy()],
            candidates,
            labels: labels
                .into_iter()
                .map(|(v, c, r)| LabelInput {
                    vacancy_id: v.into(),
                    candidate_id: c.into(),
                    relevance: r,
                })
                .collect(),
        }
    }

    #[test]
    fn missing_test_is_neutral_and_never_cited() {
        let tested = CandidateInput {
            test_result: Some(90.0),
            ..go_resume("tested")
        };
        let r = evaluate(&input(vec![go_resume("plain"), tested], vec![]), NOW).unwrap();
        let top = &r.vacancies[0].top;
        assert_eq!(top[0].candidate_id, "tested");
        let plain = &top[1];
        assert!(plain.notes.iter().any(|n| n == NO_TEST_RU));
        let test = plain
            .breakdown
            .features
            .iter()
            .find(|f| f.key == FeatureKey::Test)
            .unwrap();
        assert_eq!(test.value, NEUTRAL_TEST_STRENGTH);
        assert_eq!(test.explanation_ru, NO_TEST_RU);
        assert!(
            plain
                .breakdown
                .reasons
                .iter()
                .all(|x| !x.contains("Результат теста"))
        );
        assert!(top[0].notes.iter().all(|n| n != NO_TEST_RU));
    }

    #[test]
    fn category_origin_is_reported() {
        let given = CandidateInput {
            specialization: Some(Specialization::Backend),
            grade: Some(Grade::Middle),
            test_result: Some(50.0),
            ..go_resume("given")
        };
        let nothing = CandidateInput {
            id: "blank".into(),
            resume_text: "Бухгалтер".into(),
            ..Default::default()
        };
        let r = evaluate(
            &input(vec![go_resume("parsed"), given, nothing], vec![]),
            NOW,
        )
        .unwrap();
        let notes = |id: &str| {
            r.vacancies[0]
                .top
                .iter()
                .find(|p| p.candidate_id == id)
                .map(|p| p.notes.clone())
                .unwrap()
        };
        assert!(notes("parsed")[0].contains("из резюме, не подтверждена тестом"));
        assert!(notes("given").is_empty());
        assert_eq!(r.coverage.candidates_without_category, vec!["blank"]);
        assert_eq!(r.coverage.candidate_specialization.input, 1);
        assert_eq!(r.coverage.candidate_specialization.text, 1);
        assert_eq!(r.coverage.candidate_specialization.missing, 1);
        assert_eq!(
            r.vacancies[0].need.must_skills,
            ["Go", "PostgreSQL", "Apache Kafka"]
        );
        assert_eq!(r.now, "2026-10-01");
    }

    #[test]
    fn metrics_need_a_relevant_label() {
        let pool = vec![go_resume("a"), go_resume("b")];
        let r = evaluate(&input(pool.clone(), vec![("go", "a", 1)]), NOW).unwrap();
        assert!(r.metrics.is_none() && r.vacancies[0].ours.is_none());
        assert_eq!(r.coverage.labeled_vacancies, 1);
        let r = evaluate(&input(pool, vec![("go", "a", 3), ("go", "b", 0)]), NOW).unwrap();
        let m = r.metrics.unwrap();
        assert_eq!(m.vacancies, 1);
        assert_eq!(m.ours.judged_at_k.mean, 1.0);
        assert_eq!(r.vacancies[0].relevant_labels, 1);
    }

    #[test]
    fn ids_and_labels_are_checked() {
        let field = |i: PairsInput| evaluate(&i, NOW).unwrap_err().field();
        let pool = || vec![go_resume("a"), go_resume("b")];
        assert_eq!(
            field(input(vec![go_resume("a"), go_resume("a")], vec![])),
            "candidates"
        );
        assert_eq!(field(input(pool(), vec![("go", "x", 1)])), "labels");
        assert_eq!(field(input(pool(), vec![("nope", "a", 1)])), "labels");
        assert_eq!(
            field(input(pool(), vec![("go", "a", 1), ("go", "a", 2)])),
            "labels"
        );
        assert_eq!(field(input(pool(), vec![("go", "a", 4)])), "labels");
        let mut i = input(pool(), vec![]);
        i.now = Some("01.10.2026".into());
        assert_eq!(field(i), "now");
        let mut i = input(pool(), vec![]);
        i.k = Some(MAX_K + 1);
        assert_eq!(field(i), "k");
        let mut i = input(pool(), vec![]);
        i.vacancies = (0..=MAX_VACANCIES)
            .map(|n| VacancyInput {
                id: n.to_string(),
                ..vacancy()
            })
            .collect();
        assert!(matches!(
            evaluate(&i, NOW),
            Err(PairsError::TooMany {
                list: "vacancies",
                ..
            })
        ));
        let mut i = input(pool(), vec![]);
        i.candidates[0].test_result = Some(101.0);
        assert_eq!(field(i), "candidates");
    }

    #[test]
    fn baseline_filters_by_stated_experience() {
        assert!(years_fit(Grade::Junior, 1.0) && !years_fit(Grade::Junior, 3.0));
        assert!(years_fit(Grade::Middle, 2.0) && !years_fit(Grade::Middle, 6.0));
        assert!(years_fit(Grade::Senior, 5.0) && !years_fit(Grade::Senior, 4.9));
        let veteran = CandidateInput {
            resume_text: "Go-разработчик\nОпыт работы — 12 лет\nGo, PostgreSQL, Kafka.".into(),
            ..go_resume("veteran")
        };
        let unknown = CandidateInput {
            resume_text: "Go, PostgreSQL".into(),
            ..go_resume("unknown")
        };
        let r = evaluate(
            &input(vec![veteran, go_resume("fit"), unknown], vec![]),
            NOW,
        )
        .unwrap();
        let base: Vec<&str> = r.vacancies[0]
            .baseline_top
            .iter()
            .map(|b| b.candidate_id.as_str())
            .collect();
        assert_eq!(base, ["fit", "unknown"]);
        assert_eq!(r.vacancies[0].baseline_top[0].keywords.len(), 3);
    }

    #[test]
    fn day_parsing() {
        assert_eq!(parse_day("2026-10-01"), Some(NOW));
        assert_eq!(format_day(NOW + 3_600), "2026-10-01");
        assert_eq!(parse_day("2026-02-30"), None);
        assert_eq!(parse_day("вчера"), None);
    }
}
