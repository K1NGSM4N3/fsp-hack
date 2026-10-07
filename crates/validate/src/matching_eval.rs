//! Matching validation (architecture.md §8.2). Candidates carry a hidden truth (real skills,
//! real ability) and an observed profile (declared skills with inflation, test category with
//! measurement error, partial verification, a profile text). Relevance is judged on the truth by
//! a function that is deliberately different from the production scoring.

use std::fmt::Write;

use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use embedder::{Embedder, TextKind};
use matching::{
    Achievement, AchievementKind, CandidatePool, CandidateSkill, CandidateSnapshot, Discipline,
    EventLevel, FeatureKey, FspSummary, Need, Place, SkillSource, TextCloseness, Weights,
    pairs::terms,
    semantic::{self, Calibration, Embeddings, Query},
};
use rand::{
    Rng, SeedableRng,
    seq::{IndexedRandom, SliceRandom},
};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use matching::metrics::{self as stats, RELEVANT, Summary};

use crate::texts;

const NOW: i64 = 1_790_000_000;
const DAY: i64 = 86_400;
const POOL_PER_SPEC: usize = 400;
const NEEDS_PER_SPEC: usize = 40;
const K: usize = 10;
const MEASUREMENT_SD: f64 = 0.3;
/// Texts are drawn from their own stream, so candidates and needs are the same draws as before the
/// generator wrote texts.
const TEXT_SALT: u64 = 0x7e57_7e57;

struct Truth {
    theta: f64,
    skills: Vec<&'static str>,
}

struct Synthetic {
    truth: Truth,
    snapshot: CandidateSnapshot,
    declared: Vec<&'static str>,
    self_years: f64,
    /// Profile text as the API embeds it: role, «о себе», jobs.
    text: String,
}

fn normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn salary_base(grade: Grade) -> f64 {
    match grade {
        Grade::Junior => 120_000.0,
        Grade::Middle => 250_000.0,
        Grade::Senior => 390_000.0,
    }
}

fn best_discipline(spec: Specialization) -> Discipline {
    *Discipline::ALL
        .iter()
        .max_by(|a, b| a.relevance(spec).total_cmp(&b.relevance(spec)))
        .unwrap_or(&Discipline::Product)
}

fn candidate(
    rng: &mut ChaCha8Rng,
    text_rng: &mut ChaCha8Rng,
    spec: Specialization,
    id: usize,
) -> Option<Synthetic> {
    let theta = normal(rng);
    let measured = theta + MEASUREMENT_SD * normal(rng);
    let grade = Grade::for_theta(measured)?;

    let typical = matching::typical_skills(spec);
    let mut skills: Vec<&'static str> = typical
        .iter()
        .enumerate()
        .filter(|(i, _)| rng.random_bool(if *i < 4 { 0.65 } else { 0.35 }))
        .map(|(_, s)| *s)
        .collect();
    if skills.is_empty() {
        skills.push(typical[0]);
    }

    let mut declared: Vec<&'static str> = skills
        .iter()
        .copied()
        .filter(|_| rng.random_bool(0.85))
        .collect();
    for s in typical.iter().filter(|s| !skills.contains(s)) {
        if rng.random_bool(0.18) {
            declared.push(s);
        }
    }
    let mut observed: Vec<CandidateSkill> = declared
        .iter()
        .map(|s| CandidateSkill {
            slug: (*s).to_owned(),
            source: SkillSource::Declared,
        })
        .collect();
    for s in &skills {
        if rng.random_bool(0.45) {
            observed.push(CandidateSkill {
                slug: (*s).to_owned(),
                source: SkillSource::Test,
            });
        }
    }

    let achievements: Vec<Achievement> = if rng.random_bool(0.18) {
        let strong = theta > 0.8;
        (0..rng.random_range(1..=4))
            .map(|_| Achievement {
                kind: AchievementKind::Competition,
                discipline: best_discipline(spec),
                rank: None,
                event_name: Some("Соревнования по спортивному программированию".into()),
                event_level: Some(if strong {
                    EventLevel::National
                } else {
                    EventLevel::Regional
                }),
                place: Some(if strong && rng.random_bool(0.5) {
                    Place::Second
                } else {
                    Place::Participant
                }),
                team: None,
                role: None,
                date: format!("{}-06-01", rng.random_range(2022..=2026)),
                protocol_url: None,
            })
            .collect()
    } else {
        Vec::new()
    };
    let fsp = FspSummary::from(&matching::fsp_score(&achievements, spec, NOW));

    let true_grade = Grade::for_theta(theta).unwrap_or(Grade::Junior);
    let salary = rng
        .random_bool(0.9)
        .then(|| (salary_base(true_grade) * (0.85 + 0.35 * rng.random::<f64>())) as i64);
    let mut formats: Vec<WorkFormat> = WorkFormat::ALL
        .iter()
        .copied()
        .filter(|_| rng.random_bool(0.6))
        .collect();
    if formats.is_empty() {
        formats.push(WorkFormat::Remote);
    }
    let self_years = (1.0 + 2.5 * (theta + 1.0) + 1.5 * normal(rng)).max(0.0);
    let text = texts::profile(text_rng, spec, &skills, &declared, theta, self_years);

    Some(Synthetic {
        snapshot: CandidateSnapshot {
            id: id.to_string(),
            category: Category::new(spec, grade),
            theta: measured,
            skills: observed,
            fsp,
            salary_expectation_min: salary,
            work_formats: formats,
            city: None,
            relocation: false,
            last_activity_unix: NOW - rng.random_range(0..120) * DAY,
            industries: vec![*Industry::ALL.choose(rng).unwrap_or(&Industry::Fintech)],
            text_relevance: 0.0,
            text_by_meaning: false,
        },
        truth: Truth { theta, skills },
        declared,
        self_years,
        text,
    })
}

/// A need and its title; the description is the text a vacancy would carry.
fn need(rng: &mut ChaCha8Rng, text_rng: &mut ChaCha8Rng, spec: Specialization) -> (Need, String) {
    let grade = *[Grade::Junior, Grade::Middle, Grade::Middle, Grade::Senior]
        .choose(rng)
        .unwrap_or(&Grade::Middle);
    let mut typical: Vec<&str> = matching::typical_skills(spec).to_vec();
    let head = typical.len().min(6);
    typical[..head].shuffle(rng);
    let must_n = rng.random_range(2..=3);
    let base = salary_base(grade);
    let must: Vec<String> = typical[..must_n].iter().map(|s| (*s).to_owned()).collect();
    let nice: Vec<String> = typical[must_n..must_n + 1]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let (title, description) = texts::need(text_rng, spec, grade, &must, &nice);
    let need = Need {
        specialization: spec,
        grades: vec![grade],
        allow_adjacent_grades: rng.random_bool(0.3),
        must_skills: must,
        nice_skills: nice,
        salary_min: (base * 0.85) as i64,
        salary_max: (base * 1.2) as i64,
        work_formats: vec![*WorkFormat::ALL.choose(rng).unwrap_or(&WorkFormat::Remote)],
        city: None,
        industries: Vec::new(),
        description,
    };
    (need, title)
}

/// Hidden ground truth: real skills and real ability, never the observed profile.
fn relevance(need: &Need, c: &Synthetic) -> u8 {
    let s = &c.snapshot;
    let format_ok = s.work_formats.iter().any(|f| need.work_formats.contains(f));
    let salary_ok = s
        .salary_expectation_min
        .is_none_or(|e| e as f64 <= need.salary_max as f64 * 1.25);
    if !format_ok || !salary_ok {
        return 0;
    }
    let skill_fit = need
        .must_skills
        .iter()
        .map(|m| {
            if c.truth.skills.iter().any(|t| t == m) {
                1.0
            } else if c
                .truth
                .skills
                .iter()
                .any(|t| matching::similarity(t, m) >= 0.6)
            {
                0.5
            } else {
                0.0
            }
        })
        .sum::<f64>()
        / need.must_skills.len().max(1) as f64;
    let grade = need.grades[0];
    let cut = grade.theta_cut();
    let top = grade.next().map_or(f64::INFINITY, Grade::theta_cut);
    let t = c.truth.theta;
    let level_fit = if t >= cut && t < top + 0.5 {
        1.0
    } else if t >= top + 0.5 {
        0.6
    } else if t >= cut - 0.5 {
        0.4
    } else {
        0.0
    };
    let nice = need
        .nice_skills
        .iter()
        .filter(|n| c.truth.skills.iter().any(|t| t == *n))
        .count() as f64
        / need.nice_skills.len().max(1) as f64;
    let r = 0.5 * skill_fit + 0.45 * level_fit + 0.05 * nice;
    if level_fit == 0.0 || skill_fit < 0.34 {
        return u8::from(r >= 0.3);
    }
    match r {
        r if r >= 0.85 => 3,
        r if r >= 0.65 => 2,
        r if r >= 0.4 => 1,
        _ => 0,
    }
}

/// Classic job-board search: self-reported experience filter + keyword overlap with the resume.
fn keyword_rank(need: &Need, pool: &[Synthetic], rng: &mut ChaCha8Rng) -> Vec<usize> {
    let mut scored: Vec<(f64, usize)> = pool
        .iter()
        .enumerate()
        .filter(|(_, c)| matching::pairs::years_fit(need.grades[0], c.self_years))
        .map(|(i, c)| {
            let hits = need
                .must_skills
                .iter()
                .chain(&need.nice_skills)
                .filter(|s| c.declared.contains(&s.as_str()))
                .count();
            (hits as f64 + rng.random::<f64>() * 0.1, i)
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    scored.into_iter().map(|(_, i)| i).collect()
}

#[derive(Serialize, Clone)]
pub struct RankerResult {
    pub name: String,
    pub precision_at_10: f64,
    pub precision_ci: f64,
    pub ndcg_at_10: f64,
    pub ndcg_ci: f64,
    pub map: f64,
    pub map_ci: f64,
    pub mrr: f64,
    pub mrr_ci: f64,
}

struct Case {
    need: Need,
    pool_index: usize,
    /// Keyword closeness per candidate of the pool: shared stems of the need text and the profile,
    /// normalized by the best candidate, as the API normalizes `ts_rank_cd`.
    keywords: Vec<f32>,
    /// Cosine of the need and each profile vector; empty without a model.
    cosines: Vec<f32>,
}

/// Where the «Близость описаний» feature comes from.
#[derive(Clone, Copy)]
enum TextMode {
    /// Need without a description: the feature does not apply and its weight is redistributed.
    Off,
    /// What the API does with the embedder off.
    Keywords,
    Meaning(Calibration),
}

struct Pool {
    synthetic: Vec<Synthetic>,
    pool: CandidatePool,
    vectors: Embeddings,
}

fn evaluate<F>(name: &str, cases: &[Case], pools: &[Pool], mut ranker: F) -> RankerResult
where
    F: FnMut(&Case, &Pool) -> Vec<usize>,
{
    let (mut p, mut n, mut ap, mut m) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for case in cases {
        let pool = &pools[case.pool_index];
        let all: Vec<u8> = pool
            .synthetic
            .iter()
            .map(|c| relevance(&case.need, c))
            .collect();
        let relevant = all.iter().filter(|&&g| g >= RELEVANT).count();
        let order = ranker(case, pool);
        let ranked: Vec<u8> = order.iter().map(|&i| all[i]).collect();
        p.push(stats::precision_at(&ranked, K));
        n.push(stats::ndcg_at(&ranked, &all, K));
        ap.push(stats::average_precision(&ranked, relevant));
        m.push(stats::reciprocal_rank(&ranked[..ranked.len().min(K)]));
    }
    let (sp, sn, sa, sm): (Summary, Summary, Summary, Summary) = (
        stats::summarize(&p),
        stats::summarize(&n),
        stats::summarize(&ap),
        stats::summarize(&m),
    );
    RankerResult {
        name: name.to_owned(),
        precision_at_10: sp.mean,
        precision_ci: sp.ci95,
        ndcg_at_10: sn.mean,
        ndcg_ci: sn.ci95,
        map: sa.mean,
        map_ci: sa.ci95,
        mrr: sm.mean,
        mrr_ci: sm.ci95,
    }
}

fn closeness(case: &Case, i: usize, mode: TextMode) -> TextCloseness {
    match mode {
        TextMode::Off => TextCloseness::default(),
        TextMode::Keywords => TextCloseness::keywords(case.keywords[i]),
        TextMode::Meaning(cal) => match case.cosines.get(i).filter(|c| c.is_finite()) {
            Some(&cos) => TextCloseness::meaning(cal.closeness(cos)),
            None => TextCloseness::keywords(case.keywords[i]),
        },
    }
}

fn rank_view(case: &Case, pool: &CandidatePool, weights: &Weights, mode: TextMode) -> Vec<usize> {
    let view = pool.subset((0..pool.len()).map(|i| (i, closeness(case, i, mode))));
    let need = match mode {
        TextMode::Off => Need {
            description: String::new(),
            ..case.need.clone()
        },
        _ => case.need.clone(),
    };
    matching::rank(&need, &view, weights, NOW, 0)
        .into_iter()
        .map(|r| r.index)
        .collect()
}

fn ours(weights: Weights, mode: TextMode) -> impl FnMut(&Case, &Pool) -> Vec<usize> {
    move |case, pool| rank_view(case, &pool.pool, &weights, mode)
}

/// The same ranking over pools whose candidates declared no skills (see [`sparse_pools`]).
fn ours_sparse(
    weights: Weights,
    mode: TextMode,
    sparse: &[CandidatePool],
) -> impl FnMut(&Case, &Pool) -> Vec<usize> + '_ {
    move |case, _| rank_view(case, &sparse[case.pool_index], &weights, mode)
}

/// Candidates who filled in «о себе» and their jobs but no skill list: only skills the test
/// confirmed remain in the observed profile.
fn sparse_pools(pools: &[Pool]) -> Vec<CandidatePool> {
    pools
        .iter()
        .map(|p| {
            let snapshots: Vec<CandidateSnapshot> = p
                .synthetic
                .iter()
                .map(|c| CandidateSnapshot {
                    skills: c
                        .snapshot
                        .skills
                        .iter()
                        .filter(|s| s.source != SkillSource::Declared)
                        .cloned()
                        .collect(),
                    ..c.snapshot.clone()
                })
                .collect();
            CandidatePool::from_snapshots(&snapshots)
        })
        .collect()
}

fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 2 {
        return None;
    }
    let (mx, my) = (
        x[..n].iter().sum::<f64>() / n as f64,
        y[..n].iter().sum::<f64>() / n as f64,
    );
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x[..n].iter().zip(&y[..n]) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx) * (a - mx);
        syy += (b - my) * (b - my);
    }
    (sxx > 0.0 && syy > 0.0).then(|| sxy / (sxx * syy).sqrt())
}

/// Mean over needs of the correlation between a text closeness and how many of the need's
/// must-have skills the candidate really has.
fn signal(cases: &[Case], pools: &[Pool], values: impl Fn(&Case) -> Vec<f64>) -> Option<f64> {
    let per_need: Vec<f64> = cases
        .iter()
        .filter_map(|case| {
            let truth: Vec<f64> = pools[case.pool_index]
                .synthetic
                .iter()
                .map(|c| {
                    case.need
                        .must_skills
                        .iter()
                        .filter(|m| c.truth.skills.iter().any(|t| t == *m))
                        .count() as f64
                })
                .collect();
            pearson(&values(case), &truth)
        })
        .collect();
    (!per_need.is_empty()).then(|| per_need.iter().sum::<f64>() / per_need.len() as f64)
}

/// Semantic search alone: every candidate of the specialization by the cosine of profile and
/// need, no filters, no test, no skills.
fn by_meaning_only(case: &Case, _: &Pool) -> Vec<usize> {
    let mut order: Vec<usize> = (0..case.cosines.len()).collect();
    order.sort_by(|&a, &b| case.cosines[b].total_cmp(&case.cosines[a]).then(a.cmp(&b)));
    order
}

#[derive(Serialize)]
pub struct WeightTrial {
    pub name: String,
    pub weights: Weights,
    pub train_ndcg: f64,
}

/// How the text feature was computed in this run.
#[derive(Serialize)]
pub struct TextReport {
    /// Mean correlation of keyword closeness with the real must-have coverage, holdout needs.
    pub keywords_signal: Option<f64>,
    /// The same for the cosine of embeddings.
    pub meaning_signal: Option<f64>,
    /// `name@revision` of the embedding model, `None` when no model was available.
    pub model: Option<String>,
    /// Calibration the production code uses for this model (`matching::semantic`).
    pub calibration: Option<CalibrationReport>,
    /// Embedding time of all synthetic texts in this process, ms per text.
    pub embed_ms_per_text: Option<f64>,
}

#[derive(Serialize)]
pub struct CalibrationReport {
    pub floor: f32,
    pub ceil: f32,
    /// 10th and 99th percentiles of need × same-specialization profile cosines on the training
    /// needs: what `floor` and `ceil` were set from.
    pub fitted_floor: f32,
    pub fitted_ceil: f32,
}

#[derive(Serialize)]
pub struct MatchingReport {
    pub candidates: usize,
    pub needs_train: usize,
    pub needs_test: usize,
    pub text: TextReport,
    pub tuning: Vec<WeightTrial>,
    pub chosen: String,
    pub results: Vec<RankerResult>,
    pub text_ablation: Vec<RankerResult>,
    /// Text sources again, on profiles without declared skills.
    pub sparse_ablation: Vec<RankerResult>,
    pub ablations: Vec<RankerResult>,
}

/// The first version from architecture.md §6.3, kept to show what tuning changed.
const WEIGHTS_V1: Weights = Weights {
    must: 0.30,
    test: 0.25,
    grade: 0.10,
    fsp: 0.10,
    nice: 0.08,
    salary: 0.07,
    fresh: 0.05,
    text: 0.05,
    use_skill_graph: true,
};

/// The weights tuned before profile texts existed (text could not apply then).
const WEIGHTS_V2: Weights = Weights {
    must: 0.41,
    test: 0.10,
    grade: 0.15,
    fsp: 0.10,
    nice: 0.08,
    salary: 0.07,
    fresh: 0.05,
    text: 0.04,
    use_skill_graph: true,
};

fn weight_variants() -> Vec<(String, Weights)> {
    let d = WEIGHTS_V1;
    let v2 = WEIGHTS_V2;
    let mut out = vec![
        ("исходные (архитектура v1)".into(), d),
        ("v2: подобраны без текстов".into(), v2),
        (
            "навыки сильнее теста".into(),
            Weights {
                must: 0.38,
                test: 0.18,
                ..d
            },
        ),
        (
            "навыки ≫ тест".into(),
            Weights {
                must: 0.42,
                test: 0.14,
                grade: 0.12,
                ..d
            },
        ),
        (
            "тест сильнее навыков".into(),
            Weights {
                must: 0.22,
                test: 0.33,
                ..d
            },
        ),
        (
            "навыки ≫≫ тест".into(),
            Weights {
                must: 0.46,
                test: 0.10,
                grade: 0.12,
                ..d
            },
        ),
        (
            "грейд важнее".into(),
            Weights {
                must: 0.34,
                test: 0.18,
                grade: 0.16,
                ..d
            },
        ),
        (
            "без лишних сигналов".into(),
            Weights {
                must: 0.40,
                test: 0.22,
                grade: 0.14,
                fsp: 0.08,
                nice: 0.08,
                salary: 0.05,
                fresh: 0.03,
                text: 0.0,
                use_skill_graph: true,
            },
        ),
    ];
    // The text weight is taken from the must-have skills: both describe what the person can do.
    for text in [0.08f32, 0.12, 0.16, 0.20] {
        out.push((
            format!("v2, близость описаний {:.2}", text).replace('.', ","),
            Weights {
                must: v2.must + v2.text - text,
                text,
                ..v2
            },
        ));
    }
    out
}

fn percentile(sorted: &[f32], q: f64) -> f32 {
    if sorted.is_empty() {
        return f32::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// Embeds the texts with progress on stderr; `None` when the model fails.
fn embed_all(model: &Embedder, texts: &[&str], kind: TextKind) -> Option<Vec<Vec<f32>>> {
    let mut out = Vec::with_capacity(texts.len());
    for chunk in texts.chunks(256) {
        match model.embed(chunk, kind) {
            Ok(v) => out.extend(v),
            Err(e) => {
                eprintln!("эмбеддинги не посчитаны: {e}");
                return None;
            }
        }
        eprint!("\rэмбеддинги: {} из {}", out.len(), texts.len());
    }
    eprintln!();
    Some(out)
}

pub fn run(seed: u64, model: Option<&Embedder>) -> MatchingReport {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut text_rng = ChaCha8Rng::seed_from_u64(seed ^ TEXT_SALT);
    let mut pools: Vec<Pool> = Vec::new();
    let mut needs: Vec<(Need, String, usize, bool)> = Vec::new();
    let mut candidates = 0;
    for (pi, spec) in Specialization::ALL.iter().copied().enumerate() {
        let mut list = Vec::with_capacity(POOL_PER_SPEC);
        let mut id = 0;
        while list.len() < POOL_PER_SPEC {
            if let Some(c) = candidate(&mut rng, &mut text_rng, spec, id) {
                list.push(c);
            }
            id += 1;
        }
        candidates += list.len();
        let snapshots: Vec<CandidateSnapshot> = list.iter().map(|c| c.snapshot.clone()).collect();
        pools.push(Pool {
            pool: CandidatePool::from_snapshots(&snapshots),
            synthetic: list,
            vectors: Embeddings::default(),
        });
        for i in 0..NEEDS_PER_SPEC {
            let (n, title) = need(&mut rng, &mut text_rng, spec);
            needs.push((n, title, pi, i % 2 == 0));
        }
    }

    let started = std::time::Instant::now();
    let mut need_vectors: Vec<Option<Query>> = vec![None; needs.len()];
    let mut model_id = None;
    if let Some(m) = model {
        let profile_texts: Vec<&str> = pools
            .iter()
            .flat_map(|p| p.synthetic.iter().map(|c| c.text.as_str()))
            .collect();
        let posting: Vec<String> = needs
            .iter()
            .map(|(n, title, ..)| semantic::posting_text(title, &n.description))
            .collect();
        let posting_refs: Vec<&str> = posting.iter().map(String::as_str).collect();
        if let (Some(profiles), Some(postings)) = (
            embed_all(m, &profile_texts, TextKind::Passage),
            embed_all(m, &posting_refs, TextKind::Query),
        ) {
            let mut rows = profiles.iter();
            for p in &mut pools {
                let mut table = Embeddings::with_capacity(m.dim(), p.synthetic.len());
                for _ in 0..p.synthetic.len() {
                    table.push(rows.next().map(Vec::as_slice));
                }
                p.vectors = table;
            }
            for (slot, v) in need_vectors.iter_mut().zip(&postings) {
                *slot = Query::new(v);
            }
            model_id = Some(m.id().to_owned());
        }
    }
    let embed_ms_per_text = model_id
        .as_ref()
        .map(|_| started.elapsed().as_secs_f64() * 1000.0 / (candidates + needs.len()) as f64);

    let mut train = Vec::new();
    let mut test = Vec::new();
    let mut overlap = Vec::new();
    for ((n, title, pi, is_train), q) in needs.into_iter().zip(need_vectors) {
        let pool = &pools[pi];
        let query = terms::query(&semantic::posting_text(&title, &n.description));
        overlap.clear();
        let mut doc = Vec::new();
        for c in &pool.synthetic {
            doc.clear();
            terms::document(&c.text, &mut doc);
            overlap.push(terms::overlap(&query, &doc));
        }
        let max = overlap.iter().copied().max().unwrap_or(0);
        let keywords = overlap
            .iter()
            .map(|&o| {
                if max > 0 {
                    f32::from(o) / f32::from(max)
                } else {
                    0.0
                }
            })
            .collect();
        let cosines = match &q {
            Some(q) => (0..pool.synthetic.len())
                .map(|i| pool.vectors.cosine(i, q).unwrap_or(f32::NAN))
                .collect(),
            None => Vec::new(),
        };
        let case = Case {
            need: n,
            pool_index: pi,
            keywords,
            cosines,
        };
        if is_train {
            train.push(case);
        } else {
            test.push(case);
        }
    }

    let calibration = model_id.as_deref().and_then(semantic::calibration);
    let fitted = {
        let mut all: Vec<f32> = train
            .iter()
            .flat_map(|c| c.cosines.iter().copied())
            .filter(|c| c.is_finite())
            .collect();
        all.sort_by(f32::total_cmp);
        (percentile(&all, 0.10), percentile(&all, 0.99))
    };
    let production = match calibration {
        Some(cal) => TextMode::Meaning(cal),
        None => TextMode::Keywords,
    };

    let mut tuning: Vec<WeightTrial> = weight_variants()
        .into_iter()
        .map(|(name, w)| {
            let r = evaluate(&name, &train, &pools, ours(w, production));
            WeightTrial {
                name,
                weights: w,
                train_ndcg: r.ndcg_at_10,
            }
        })
        .collect();
    tuning.sort_by(|a, b| b.train_ndcg.total_cmp(&a.train_ndcg));
    let best = tuning[0].weights;
    let chosen = tuning[0].name.clone();

    let mut baseline_rng = ChaCha8Rng::seed_from_u64(seed ^ 0xBADC0DE);
    let mut random_rng = ChaCha8Rng::seed_from_u64(seed ^ 0xFACE);
    let production_name = match production {
        TextMode::Meaning(_) => "близость описаний по смыслу",
        _ => "близость описаний по ключевым словам",
    };
    let mut results = vec![
        evaluate(
            &format!("Наш подбор (веса: {chosen}; {production_name})"),
            &test,
            &pools,
            ours(best, production),
        ),
        evaluate(
            "Наш подбор, исходные веса v1",
            &test,
            &pools,
            ours(WEIGHTS_V1, production),
        ),
        evaluate(
            "Ключевые слова резюме + заявленный стаж",
            &test,
            &pools,
            |c, p| keyword_rank(&c.need, &p.synthetic, &mut baseline_rng),
        ),
    ];
    if model_id.is_some() {
        results.push(evaluate(
            "Только близость описаний по смыслу (семантический поиск)",
            &test,
            &pools,
            by_meaning_only,
        ));
    }
    results.push(evaluate(
        "Случайный порядок в специализации",
        &test,
        &pools,
        |_, p| {
            let mut v: Vec<usize> = (0..p.synthetic.len()).collect();
            v.shuffle(&mut random_rng);
            v
        },
    ));

    let mut text_ablation = Vec::new();
    if let TextMode::Meaning(_) = production {
        text_ablation.push(evaluate(
            "по смыслу (эмбеддинги)",
            &test,
            &pools,
            ours(best, production),
        ));
    }
    text_ablation.push(evaluate(
        "по ключевым словам (эмбеддер выключен)",
        &test,
        &pools,
        ours(best, TextMode::Keywords),
    ));
    text_ablation.push(evaluate(
        "признак не применяется (описания нет)",
        &test,
        &pools,
        ours(best, TextMode::Off),
    ));

    let sparse = sparse_pools(&pools);
    let mut sparse_ablation = Vec::new();
    if let TextMode::Meaning(_) = production {
        sparse_ablation.push(evaluate(
            "по смыслу (эмбеддинги)",
            &test,
            &pools,
            ours_sparse(best, production, &sparse),
        ));
    }
    sparse_ablation.push(evaluate(
        "по ключевым словам",
        &test,
        &pools,
        ours_sparse(best, TextMode::Keywords, &sparse),
    ));
    sparse_ablation.push(evaluate(
        "признак не применяется",
        &test,
        &pools,
        ours_sparse(best, TextMode::Off, &sparse),
    ));
    let keywords_signal = signal(&test, &pools, |c| {
        c.keywords.iter().map(|&v| f64::from(v)).collect()
    });
    let meaning_signal = model_id.as_ref().and_then(|_| {
        signal(&test, &pools, |c| {
            c.cosines.iter().map(|&v| f64::from(v)).collect()
        })
    });

    let ablations = vec![
        evaluate(
            "без результата теста",
            &test,
            &pools,
            ours(best.without(FeatureKey::Test), production),
        ),
        evaluate(
            "без достижений ФСП",
            &test,
            &pools,
            ours(best.without(FeatureKey::Fsp), production),
        ),
        evaluate(
            "без графа близости навыков",
            &test,
            &pools,
            ours(
                Weights {
                    use_skill_graph: false,
                    ..best
                },
                production,
            ),
        ),
        evaluate(
            "только навыки (must + nice)",
            &test,
            &pools,
            ours(
                Weights {
                    must: 0.8,
                    nice: 0.2,
                    test: 0.0,
                    grade: 0.0,
                    fsp: 0.0,
                    salary: 0.0,
                    fresh: 0.0,
                    text: 0.0,
                    use_skill_graph: true,
                },
                production,
            ),
        ),
    ];
    MatchingReport {
        candidates,
        needs_train: train.len(),
        needs_test: test.len(),
        text: TextReport {
            keywords_signal,
            meaning_signal,
            calibration: calibration.map(|c| CalibrationReport {
                floor: c.floor,
                ceil: c.ceil,
                fitted_floor: fitted.0,
                fitted_ceil: fitted.1,
            }),
            model: model_id,
            embed_ms_per_text,
        },
        tuning,
        chosen,
        results,
        text_ablation,
        sparse_ablation,
        ablations,
    }
}

pub fn markdown(r: &MatchingReport) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "## Подбор: доля релевантных в топе\n");
    let _ = writeln!(
        s,
        "Синтетика: {} кандидатов (6 специализаций × 400), {} потребностей для подбора весов и {} отложенных для оценки. \
         У каждого кандидата есть скрытая истина (реальные навыки и способность θ) и наблюдаемый профиль: \
         заявленные навыки с приукрашиванием (~18 % лишних), пропуски (~15 % реальных не заявлены), категория из теста \
         с ошибкой измерения σ = {MEASUREMENT_SD}, подтверждённые тестом навыки (~45 % реальных), ФСП у ~18 %. \
         Релевантность (0–3) считается по истине функцией, отличной от скоринга: реальное покрытие обязательных навыков, \
         попадание реальной θ в диапазон грейда, совместимость формата и зарплаты. Релевантным считается оценка ≥ 2.\n",
        r.candidates, r.needs_train, r.needs_test
    );
    let _ = writeln!(
        s,
        "Тексты тоже синтетические (`crates/validate/src/texts.rs`). В «о себе» — заявленный стек, с тем же \
         приукрашиванием; в описаниях мест работы — задачи по реальным навыкам (каждый реальный навык описан с \
         вероятностью 0,75), чаще всего без названия технологии в том виде, в каком его пишет вакансия; у половины — \
         фраза об уровне по реальной θ. Описание потребности пересказывает её обязательные и желательные навыки \
         своими словами. Отрасли кандидата и потребности выбираются независимо — для любого текстового метода это шум. \
         Это допущения генератора: он показывает, что даёт смысловая близость, когда описание опыта отражает реальную \
         работу, а не измеряет рынок.\n"
    );
    match (&r.text.model, &r.text.calibration) {
        (Some(model), Some(c)) => {
            let _ = writeln!(
                s,
                "Близость описаний по смыслу: модель `{model}`, косинус переводится в 0–1 линейно между {:.2} и {:.2} \
                 (`matching::semantic`); на обучающих потребностях 10-й и 99-й перцентили косинуса — {:.3} и {:.3}. \
                 Профили и потребности кодируются тем же квантованием в `i8`, что и в API.{}\n",
                c.floor,
                c.ceil,
                c.fitted_floor,
                c.fitted_ceil,
                r.text
                    .embed_ms_per_text
                    .map(|ms| format!(
                        " Эмбеддинги синтетики в этом прогоне — {ms:.1} мс на текст."
                    ))
                    .unwrap_or_default()
            );
        }
        (Some(model), None) => {
            let _ = writeln!(
                s,
                "Модель `{model}` не откалибрована в `matching::semantic`: близость описаний считается по ключевым словам.\n"
            );
        }
        _ => {
            let _ = writeln!(
                s,
                "Модели эмбеддингов не нашлось (`make model`): близость описаний — по ключевым словам, как в API при \
                 выключенном эмбеддере.\n"
            );
        }
    }
    let table = |s: &mut String, rows: &[RankerResult]| {
        let _ = writeln!(
            s,
            "| Ранжирование | Precision@10 | nDCG@10 | MAP | MRR |\n|---|---|---|---|---|"
        );
        for x in rows {
            let _ = writeln!(
                s,
                "| {} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} |",
                x.name,
                x.precision_at_10,
                x.precision_ci,
                x.ndcg_at_10,
                x.ndcg_ci,
                x.map,
                x.map_ci,
                x.mrr,
                x.mrr_ci
            );
        }
    };
    let _ = writeln!(s, "### Результаты на отложенных потребностях\n");
    table(&mut s, &r.results);
    let _ = writeln!(
        s,
        "\n### Откуда близость описаний (веса: {}, те же потребности)\n",
        r.chosen
    );
    table(&mut s, &r.text_ablation);
    let signal = |v: Option<f64>| v.map_or_else(|| "—".to_owned(), |v| format!("{v:.2}"));
    let _ = writeln!(
        s,
        "\nКорреляция близости с тем, сколькими обязательными навыками потребности кандидат владеет на самом деле \
         (среднее по отложенным потребностям): по ключевым словам — {}, косинус эмбеддингов — {}. Сигнал в текстах \
         есть, но его почти целиком перекрывают заявленные и подтверждённые тестом навыки.",
        signal(r.text.keywords_signal),
        signal(r.text.meaning_signal)
    );
    let _ = writeln!(
        s,
        "\n### Профили без заявленных навыков (те же веса и потребности)\n\nСтресс-сценарий: кандидат заполнил «о себе» \
         и места работы, но не список навыков — в наблюдаемом профиле остаются только навыки, подтверждённые тестом. \
         Веса на нём не подбирались.\n"
    );
    table(&mut s, &r.sparse_ablation);
    let _ = writeln!(s, "\n### Абляции (веса: {})\n", r.chosen);
    table(&mut s, &r.ablations);
    let _ = writeln!(
        s,
        "\n### Подбор весов (nDCG@10 на обучающих потребностях)\n\n| Вариант | must | test | grade | fsp | nice | salary | fresh | text | nDCG@10 |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for t in &r.tuning {
        let w = &t.weights;
        let _ = writeln!(
            s,
            "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} |",
            t.name, w.must, w.test, w.grade, w.fsp, w.nice, w.salary, w.fresh, w.text, t.train_ndcg
        );
    }
    let _ = writeln!(
        s,
        "\n± — полуширина 95 % доверительного интервала по потребностям. MAP — средняя точность по всем \
         релевантным кандидатам специализации, MRR — по первой десятке."
    );
    s
}
