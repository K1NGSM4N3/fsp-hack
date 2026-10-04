//! Matching validation (architecture.md §8.2). Candidates carry a hidden truth (real skills,
//! real ability) and an observed profile (declared skills with inflation, test category with
//! measurement error, partial verification). Relevance is judged on the truth by a function
//! that is deliberately different from the production scoring.

use std::fmt::Write;

use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use matching::{
    Achievement, AchievementKind, CandidatePool, CandidateSkill, CandidateSnapshot, Discipline,
    EventLevel, FeatureKey, FspSummary, Need, Place, SkillSource, Weights,
};
use rand::{
    Rng, SeedableRng,
    seq::{IndexedRandom, SliceRandom},
};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::stats::{self, Summary};

const NOW: i64 = 1_790_000_000;
const DAY: i64 = 86_400;
const POOL_PER_SPEC: usize = 400;
const NEEDS_PER_SPEC: usize = 40;
const K: usize = 10;
const MEASUREMENT_SD: f64 = 0.3;

struct Truth {
    theta: f64,
    skills: Vec<&'static str>,
}

struct Synthetic {
    truth: Truth,
    snapshot: CandidateSnapshot,
    declared: Vec<&'static str>,
    self_years: f64,
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

fn candidate(rng: &mut ChaCha8Rng, spec: Specialization, id: usize) -> Option<Synthetic> {
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
        },
        truth: Truth { theta, skills },
        declared,
        self_years,
    })
}

fn need(rng: &mut ChaCha8Rng, spec: Specialization) -> Need {
    let grade = *[Grade::Junior, Grade::Middle, Grade::Middle, Grade::Senior]
        .choose(rng)
        .unwrap_or(&Grade::Middle);
    let mut typical: Vec<&str> = matching::typical_skills(spec).to_vec();
    let head = typical.len().min(6);
    typical[..head].shuffle(rng);
    let must_n = rng.random_range(2..=3);
    let base = salary_base(grade);
    Need {
        specialization: spec,
        grades: vec![grade],
        allow_adjacent_grades: rng.random_bool(0.3),
        must_skills: typical[..must_n].iter().map(|s| (*s).to_owned()).collect(),
        nice_skills: typical[must_n..must_n + 1]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
        salary_min: (base * 0.85) as i64,
        salary_max: (base * 1.2) as i64,
        work_formats: vec![*WorkFormat::ALL.choose(rng).unwrap_or(&WorkFormat::Remote)],
        city: None,
        industries: Vec::new(),
        description: String::new(),
    }
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

fn years_fit(grade: Grade, years: f64) -> bool {
    match grade {
        Grade::Junior => years < 2.5,
        Grade::Middle => (2.0..6.0).contains(&years),
        Grade::Senior => years >= 5.0,
    }
}

/// Classic job-board search: self-reported experience filter + keyword overlap with the resume.
fn keyword_rank(need: &Need, pool: &[Synthetic], rng: &mut ChaCha8Rng) -> Vec<usize> {
    let mut scored: Vec<(f64, usize)> = pool
        .iter()
        .enumerate()
        .filter(|(_, c)| years_fit(need.grades[0], c.self_years))
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
    pub mrr: f64,
    pub mrr_ci: f64,
}

struct Case {
    need: Need,
    pool_index: usize,
}

fn evaluate<F>(
    name: &str,
    cases: &[Case],
    pools: &[(Vec<Synthetic>, CandidatePool)],
    mut ranker: F,
) -> RankerResult
where
    F: FnMut(&Need, &[Synthetic], &CandidatePool) -> Vec<usize>,
{
    let (mut p, mut n, mut m) = (Vec::new(), Vec::new(), Vec::new());
    for case in cases {
        let (synthetic, pool) = &pools[case.pool_index];
        let all: Vec<u8> = synthetic.iter().map(|c| relevance(&case.need, c)).collect();
        let order = ranker(&case.need, synthetic, pool);
        let ranked: Vec<u8> = order.iter().take(K).map(|&i| all[i]).collect();
        p.push(stats::precision_at(&ranked, K));
        n.push(stats::ndcg_at(&ranked, &all, K));
        m.push(stats::reciprocal_rank(&ranked));
    }
    let (sp, sn, sm): (Summary, Summary, Summary) = (
        stats::summarize(&p),
        stats::summarize(&n),
        stats::summarize(&m),
    );
    RankerResult {
        name: name.to_owned(),
        precision_at_10: sp.mean,
        precision_ci: sp.ci95,
        ndcg_at_10: sn.mean,
        ndcg_ci: sn.ci95,
        mrr: sm.mean,
        mrr_ci: sm.ci95,
    }
}

fn ours(weights: Weights) -> impl FnMut(&Need, &[Synthetic], &CandidatePool) -> Vec<usize> {
    move |need, _, pool| {
        matching::rank(need, pool, &weights, NOW, 0)
            .into_iter()
            .map(|r| r.index)
            .collect()
    }
}

#[derive(Serialize)]
pub struct WeightTrial {
    pub name: String,
    pub weights: Weights,
    pub train_ndcg: f64,
}

#[derive(Serialize)]
pub struct MatchingReport {
    pub candidates: usize,
    pub needs_train: usize,
    pub needs_test: usize,
    pub tuning: Vec<WeightTrial>,
    pub chosen: String,
    pub results: Vec<RankerResult>,
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

fn weight_variants() -> Vec<(String, Weights)> {
    let d = WEIGHTS_V1;
    vec![
        ("исходные (архитектура v1)".into(), d),
        ("текущие по умолчанию".into(), Weights::default()),
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
    ]
}

pub fn run(seed: u64) -> MatchingReport {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut pools: Vec<(Vec<Synthetic>, CandidatePool)> = Vec::new();
    let mut train = Vec::new();
    let mut test = Vec::new();
    let mut candidates = 0;
    for (pi, spec) in Specialization::ALL.iter().copied().enumerate() {
        let mut list = Vec::with_capacity(POOL_PER_SPEC);
        let mut id = 0;
        while list.len() < POOL_PER_SPEC {
            if let Some(c) = candidate(&mut rng, spec, id) {
                list.push(c);
            }
            id += 1;
        }
        candidates += list.len();
        let snapshots: Vec<CandidateSnapshot> = list.iter().map(|c| c.snapshot.clone()).collect();
        pools.push((list, CandidatePool::from_snapshots(&snapshots)));
        for i in 0..NEEDS_PER_SPEC {
            let case = Case {
                need: need(&mut rng, spec),
                pool_index: pi,
            };
            if i % 2 == 0 {
                train.push(case)
            } else {
                test.push(case)
            }
        }
    }

    let mut tuning: Vec<WeightTrial> = weight_variants()
        .into_iter()
        .map(|(name, w)| {
            let r = evaluate(&name, &train, &pools, ours(w));
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
    let results = vec![
        evaluate(
            &format!("Наш подбор (веса: {chosen})"),
            &test,
            &pools,
            ours(best),
        ),
        evaluate(
            "Наш подбор, исходные веса v1",
            &test,
            &pools,
            ours(WEIGHTS_V1),
        ),
        evaluate(
            "Ключевые слова резюме + заявленный стаж",
            &test,
            &pools,
            |n, s, _| keyword_rank(n, s, &mut baseline_rng),
        ),
        evaluate(
            "Случайный порядок в специализации",
            &test,
            &pools,
            |_, s, _| {
                let mut v: Vec<usize> = (0..s.len()).collect();
                v.shuffle(&mut random_rng);
                v
            },
        ),
    ];
    let ablations = vec![
        evaluate(
            "без результата теста",
            &test,
            &pools,
            ours(best.without(FeatureKey::Test)),
        ),
        evaluate(
            "без достижений ФСП",
            &test,
            &pools,
            ours(best.without(FeatureKey::Fsp)),
        ),
        evaluate(
            "без графа близости навыков",
            &test,
            &pools,
            ours(Weights {
                use_skill_graph: false,
                ..best
            }),
        ),
        evaluate(
            "только навыки (must + nice)",
            &test,
            &pools,
            ours(Weights {
                must: 0.8,
                nice: 0.2,
                test: 0.0,
                grade: 0.0,
                fsp: 0.0,
                salary: 0.0,
                fresh: 0.0,
                text: 0.0,
                use_skill_graph: true,
            }),
        ),
    ];
    MatchingReport {
        candidates,
        needs_train: train.len(),
        needs_test: test.len(),
        tuning,
        chosen,
        results,
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
    let table = |s: &mut String, rows: &[RankerResult]| {
        let _ = writeln!(
            s,
            "| Ранжирование | Precision@10 | nDCG@10 | MRR |\n|---|---|---|---|"
        );
        for x in rows {
            let _ = writeln!(
                s,
                "| {} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} |",
                x.name, x.precision_at_10, x.precision_ci, x.ndcg_at_10, x.ndcg_ci, x.mrr, x.mrr_ci
            );
        }
    };
    let _ = writeln!(s, "### Результаты на отложенных потребностях\n");
    table(&mut s, &r.results);
    let _ = writeln!(s, "\n### Абляции (веса: {})\n", r.chosen);
    table(&mut s, &r.ablations);
    let _ = writeln!(
        s,
        "\n### Подбор весов (nDCG@10 на обучающих потребностях)\n\n| Вариант | must | test | grade | fsp | nice | salary | fresh | nDCG@10 |\n|---|---|---|---|---|---|---|---|---|"
    );
    for t in &r.tuning {
        let w = &t.weights;
        let _ = writeln!(
            s,
            "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} |",
            t.name, w.must, w.test, w.grade, w.fsp, w.nice, w.salary, w.fresh, t.train_ndcg
        );
    }
    let _ = writeln!(
        s,
        "\n± — полуширина 95 % доверительного интервала по потребностям."
    );
    s
}
