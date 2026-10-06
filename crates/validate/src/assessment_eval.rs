//! Assessment validation (architecture.md §8.1) on the production engine: the same `assessment`
//! crate drives every simulated session; only the responses are synthetic (3PL at a true θ).

use std::{
    collections::HashSet,
    fmt::Write,
    hash::{DefaultHasher, Hash, Hasher},
};

use assessment::{
    AttemptKind, GradePolicy, ItemParams, ItemRef, Outcome, Posterior, SessionState,
    policy::Attempt,
};
use catalog::{Grade, Specialization};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

const PER_SPEC: usize = 400;
const WEAK_PER_SPEC: usize = 300;
const LEAK_DB_SESSIONS: usize = 600;
const FIXED_FORM_LEN: usize = 15;

fn normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Item parameters the "real world" uses. With `misspecified`, every template level drifts from
/// the engine's prior calibration (a ×0.8…1.2, b ±0.3) — the engine still uses nominal values.
fn true_params(item: &ItemRef, nominal: ItemParams, misspecified: bool) -> ItemParams {
    if !misspecified {
        return nominal;
    }
    let mut h = DefaultHasher::new();
    (item.template_id.as_str(), item.level).hash(&mut h);
    let mut r = ChaCha8Rng::seed_from_u64(h.finish());
    ItemParams {
        a: nominal.a * r.random_range(0.8..1.2),
        b: nominal.b + r.random_range(-0.3..0.3),
        c: nominal.c,
    }
}

fn instance_hash(item: &ItemRef) -> u64 {
    let mut h = DefaultHasher::new();
    if let Ok(v) = assessment::render(item) {
        v.template_id.hash(&mut h);
        v.prompt.hash(&mut h);
        v.code.as_ref().map(|c| c.source.as_str()).hash(&mut h);
        v.options.hash(&mut h);
    }
    h.finish()
}

struct Session {
    state: SessionState,
    decision: assessment::Decision,
}

/// One CAT session. `leak` answers correctly whenever the rendered instance was seen before.
fn run_session(
    rng: &mut ChaCha8Rng,
    spec: Specialization,
    claimed: Grade,
    theta: f64,
    misspecified: bool,
    leak: Option<&HashSet<u64>>,
    hits: &mut (usize, usize),
) -> Option<Session> {
    let bank = assessment::bank();
    let mut state = assessment::start(spec, claimed, rng.random(), Vec::new());
    loop {
        if let Some(decision) = assessment::decision(&state) {
            return Some(Session { state, decision });
        }
        let item = assessment::next_item(&state)?;
        let p = true_params(&item, bank.params(&item)?, misspecified);
        let leaked = leak.is_some_and(|db| db.contains(&instance_hash(&item)));
        hits.1 += 1;
        hits.0 += usize::from(leaked);
        let correct = leaked || rng.random::<f64>() < assessment::irt::p_correct(theta, &p);
        assessment::record(&mut state, item, correct, 30_000).ok()?;
    }
}

/// Placement as the API runs it: the claimed grade, then every step down the production
/// `GradePolicy` allows right after a fail (Senior → Middle → Junior), one session an hour.
fn place(
    rng: &mut ChaCha8Rng,
    spec: Specialization,
    claimed: Grade,
    theta: f64,
    misspecified: bool,
) -> (Option<Grade>, Vec<Session>) {
    const HOUR: i64 = 3_600;
    let policy = GradePolicy::default();
    let mut sessions = Vec::new();
    let mut attempts: Vec<Attempt> = Vec::new();
    let mut hits = (0, 0);
    let (mut grade, mut kind) = (claimed, AttemptKind::Placement);
    loop {
        let Some(s) = run_session(rng, spec, grade, theta, misspecified, None, &mut hits) else {
            return (None, sessions);
        };
        let passed = s.decision.passed;
        sessions.push(s);
        attempts.push(Attempt {
            specialization: spec,
            grade,
            kind,
            outcome: if passed {
                Outcome::Passed
            } else {
                Outcome::Failed
            },
            assigned: passed,
            at: attempts.len() as i64 * HOUR,
        });
        if passed {
            return (Some(grade), sessions);
        }
        let now = attempts.len() as i64 * HOUR;
        let next = grade
            .prev()
            .map(|lower| policy.check(spec, &attempts, AttemptKind::PlacementStepDown, lower, now));
        match next {
            Some(Ok(a)) => (grade, kind) = (a.grade, a.kind),
            _ => return (None, sessions),
        }
    }
}

fn grade_index(g: Option<Grade>) -> usize {
    g.map_or(0, |g| g as usize + 1)
}

const GRADE_LABELS: [&str; 4] = ["нет категории", "Junior", "Middle", "Senior"];

fn kappa(m: &[[u32; 4]; 4]) -> f64 {
    let n: f64 = m.iter().flatten().map(|&v| f64::from(v)).sum();
    if n == 0.0 {
        return 0.0;
    }
    let po = (0..4).map(|i| f64::from(m[i][i])).sum::<f64>() / n;
    let pe = (0..4)
        .map(|i| {
            let row: f64 = m[i].iter().map(|&v| f64::from(v)).sum();
            let col: f64 = (0..4).map(|r| f64::from(m[r][i])).sum();
            row * col
        })
        .sum::<f64>()
        / (n * n);
    if pe >= 1.0 {
        1.0
    } else {
        (po - pe) / (1.0 - pe)
    }
}

#[derive(Serialize, Default, Clone)]
pub struct Scenario {
    pub name: String,
    pub decision_accuracy: f64,
    pub final_grade_accuracy: f64,
    pub within_one_grade: f64,
    pub retest_agreement: f64,
    pub retest_kappa: f64,
    pub theta_rmse: f64,
    pub theta_bias: f64,
    pub mean_items: f64,
    pub confusion: [[u32; 4]; 4],
}

fn scenario(seed: u64, name: &str, misspecified: bool, noisy_claims: bool) -> Scenario {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut confusion = [[0u32; 4]; 4];
    let mut retest = [[0u32; 4]; 4];
    let (mut decisions_ok, mut decisions) = (0usize, 0usize);
    let (mut sq, mut bias, mut n_theta, mut items) = (0.0, 0.0, 0usize, 0usize);
    let (mut sessions_total, mut within) = (0usize, 0usize);
    let mut people = 0usize;
    for spec in Specialization::ALL.iter().copied() {
        for _ in 0..PER_SPEC {
            let theta = normal(&mut rng).clamp(-2.5, 2.8);
            let truth = Grade::for_theta(theta);
            let honest = truth.unwrap_or(Grade::Junior);
            let claimed = if noisy_claims {
                match rng.random_range(0..10) {
                    0..=1 => honest.next().unwrap_or(honest),
                    2 => honest.prev().unwrap_or(honest),
                    _ => honest,
                }
            } else {
                honest
            };
            let (assigned, sessions) = place(&mut rng, spec, claimed, theta, misspecified);
            let (again, _) = place(&mut rng, spec, claimed, theta, misspecified);
            confusion[grade_index(truth)][grade_index(assigned)] += 1;
            retest[grade_index(assigned)][grade_index(again)] += 1;
            within += usize::from(grade_index(truth).abs_diff(grade_index(assigned)) <= 1);
            people += 1;
            for s in &sessions {
                let cut = s.state.claimed_grade.theta_cut();
                decisions += 1;
                decisions_ok += usize::from(s.decision.passed == (theta >= cut));
                let err = s.decision.theta - theta;
                sq += err * err;
                bias += err;
                n_theta += 1;
                items += usize::from(s.decision.items_administered);
                sessions_total += 1;
            }
        }
    }
    let diag: u32 = (0..4).map(|i| confusion[i][i]).sum();
    let agree: u32 = (0..4).map(|i| retest[i][i]).sum();
    Scenario {
        name: name.to_owned(),
        decision_accuracy: decisions_ok as f64 / decisions.max(1) as f64,
        final_grade_accuracy: f64::from(diag) / people as f64,
        within_one_grade: within as f64 / people as f64,
        retest_agreement: f64::from(agree) / people as f64,
        retest_kappa: kappa(&retest),
        theta_rmse: (sq / n_theta.max(1) as f64).sqrt(),
        theta_bias: bias / n_theta.max(1) as f64,
        mean_items: items as f64 / sessions_total.max(1) as f64,
        confusion,
    }
}

#[derive(Serialize)]
pub struct TemplateStat {
    pub template_id: String,
    pub shown: u32,
    pub point_biserial: f64,
}

#[derive(Serialize)]
pub struct ItemQuality {
    pub templates: Vec<TemplateStat>,
    pub mean_point_biserial: f64,
    pub share_above_02: f64,
    pub max_exposure: f64,
    pub median_exposure: f64,
}

fn point_biserial(rows: &[(bool, f64)]) -> f64 {
    let n = rows.len() as f64;
    let ones: Vec<f64> = rows.iter().filter(|r| r.0).map(|r| r.1).collect();
    let zeros: Vec<f64> = rows.iter().filter(|r| !r.0).map(|r| r.1).collect();
    if ones.is_empty() || zeros.is_empty() {
        return 0.0;
    }
    let mean = rows.iter().map(|r| r.1).sum::<f64>() / n;
    let sd = (rows.iter().map(|r| (r.1 - mean).powi(2)).sum::<f64>() / n).sqrt();
    let m1 = ones.iter().sum::<f64>() / ones.len() as f64;
    let m0 = zeros.iter().sum::<f64>() / zeros.len() as f64;
    let p = ones.len() as f64 / n;
    if sd == 0.0 {
        0.0
    } else {
        (m1 - m0) / sd * (p * (1.0 - p)).sqrt()
    }
}

struct TemplateRows {
    id: String,
    rows: Vec<(bool, f64)>,
    shown: u32,
}

fn item_quality(seed: u64) -> ItemQuality {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut by_template: Vec<TemplateRows> = Vec::new();
    let mut exposure: Vec<f64> = Vec::new();
    for spec in Specialization::ALL.iter().copied() {
        let mut shown: Vec<(String, u32)> = Vec::new();
        let mut sessions = 0u32;
        for _ in 0..PER_SPEC {
            let theta = normal(&mut rng).clamp(-2.5, 2.8);
            let claimed = Grade::for_theta(theta).unwrap_or(Grade::Junior);
            let mut hits = (0, 0);
            let Some(s) = run_session(&mut rng, spec, claimed, theta, false, None, &mut hits)
            else {
                continue;
            };
            sessions += 1;
            for a in &s.state.administered {
                let id = a.item.template_id.clone();
                match by_template.iter_mut().find(|t| t.id == id) {
                    Some(t) => {
                        t.rows.push((a.correct, theta));
                        t.shown += 1;
                    }
                    None => by_template.push(TemplateRows {
                        id: id.clone(),
                        rows: vec![(a.correct, theta)],
                        shown: 1,
                    }),
                }
                match shown.iter_mut().find(|t| t.0 == id) {
                    Some(t) => t.1 += 1,
                    None => shown.push((id, 1)),
                }
            }
        }
        exposure.extend(
            shown
                .iter()
                .map(|(_, n)| f64::from(*n) / f64::from(sessions.max(1))),
        );
    }
    let mut templates: Vec<TemplateStat> = by_template
        .into_iter()
        .map(|t| TemplateStat {
            point_biserial: point_biserial(&t.rows),
            template_id: t.id,
            shown: t.shown,
        })
        .collect();
    templates.sort_by(|a, b| a.point_biserial.total_cmp(&b.point_biserial));
    exposure.sort_by(f64::total_cmp);
    let n = templates.len().max(1) as f64;
    ItemQuality {
        mean_point_biserial: templates.iter().map(|t| t.point_biserial).sum::<f64>() / n,
        share_above_02: templates.iter().filter(|t| t.point_biserial >= 0.2).count() as f64 / n,
        max_exposure: exposure.last().copied().unwrap_or(0.0),
        median_exposure: exposure.get(exposure.len() / 2).copied().unwrap_or(0.0),
        templates,
    }
}

#[derive(Serialize)]
pub struct LeakRow {
    pub leak_share: f64,
    pub fixed_form_pass_rate: f64,
    pub fixed_form_flagged: f64,
    pub cat_pass_rate: f64,
    pub cat_collision_rate: f64,
}

#[derive(Serialize)]
pub struct LeakReport {
    pub rows: Vec<LeakRow>,
    pub honest_flagged_fixed: f64,
    pub leak_db_instances: usize,
}

fn fixed_form(spec: Specialization) -> Vec<(ItemRef, ItemParams)> {
    let bank = assessment::bank();
    let mut candidates: Vec<(f64, ItemRef, ItemParams)> = Vec::new();
    for t in bank.for_specialization(spec) {
        let best = t
            .levels
            .iter()
            .map(|l| (assessment::irt::information(0.0, &l.params), l))
            .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((info, l)) = best {
            candidates.push((
                info,
                ItemRef {
                    template_id: t.id.to_owned(),
                    level: l.level,
                    seed: 1,
                },
                l.params,
            ));
        }
    }
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    candidates
        .into_iter()
        .take(FIXED_FORM_LEN)
        .map(|(_, r, p)| (r, p))
        .collect()
}

/// Weak candidates (true θ below the middle cut) take a middle test after answers leaked.
/// Fixed form: one public form, `share` of its answers are known. CAT: the leak database holds
/// every instance shown in earlier sessions, a leaker answers right whenever an instance repeats.
fn leak(seed: u64) -> LeakReport {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let shares = [0.0, 0.25, 0.5, 1.0];
    let mut rows: Vec<LeakRow> = shares
        .iter()
        .map(|&s| LeakRow {
            leak_share: s,
            fixed_form_pass_rate: 0.0,
            fixed_form_flagged: 0.0,
            cat_pass_rate: 0.0,
            cat_collision_rate: 0.0,
        })
        .collect();
    let mut honest_flagged = 0usize;
    let mut db_size = 0usize;
    let mut people = 0usize;
    for spec in Specialization::ALL.iter().copied() {
        let form = fixed_form(spec);
        let mut db: HashSet<u64> = HashSet::new();
        for _ in 0..LEAK_DB_SESSIONS {
            let theta = normal(&mut rng);
            let mut hits = (0, 0);
            if let Some(s) =
                run_session(&mut rng, spec, Grade::Middle, theta, false, None, &mut hits)
            {
                db.extend(s.state.administered.iter().map(|a| instance_hash(&a.item)));
            }
        }
        db_size += db.len();
        for _ in 0..WEAK_PER_SPEC {
            let theta = rng.random_range(-0.9..-0.1);
            people += 1;
            for row in rows.iter_mut() {
                let known = (row.leak_share * form.len() as f64).round() as usize;
                let mut post = Posterior::new(Grade::Middle.theta_cut());
                let mut answers = Vec::with_capacity(form.len());
                for (i, (_, p)) in form.iter().enumerate() {
                    let correct =
                        i < known || rng.random::<f64>() < assessment::irt::p_correct(theta, p);
                    post.update(p, correct);
                    answers.push((p, correct));
                }
                let lz = assessment::irt::person_fit_lz(
                    post.mean(),
                    answers.iter().map(|(p, c)| (*p, *c)),
                );
                row.fixed_form_pass_rate +=
                    f64::from(u8::from(post.mean() >= Grade::Middle.theta_cut()));
                let flagged = lz < -2.0;
                row.fixed_form_flagged += f64::from(u8::from(flagged));
                if row.leak_share == 0.0 {
                    honest_flagged += usize::from(flagged);
                }

                let mut hits = (0, 0);
                let leak_db = (row.leak_share > 0.0).then_some(&db);
                if let Some(s) = run_session(
                    &mut rng,
                    spec,
                    Grade::Middle,
                    theta,
                    false,
                    leak_db,
                    &mut hits,
                ) {
                    row.cat_pass_rate += f64::from(u8::from(s.decision.passed));
                }
                if hits.1 > 0 {
                    row.cat_collision_rate += hits.0 as f64 / hits.1 as f64;
                }
            }
        }
    }
    for row in &mut rows {
        let n = people.max(1) as f64;
        row.fixed_form_pass_rate /= n;
        row.fixed_form_flagged /= n;
        row.cat_pass_rate /= n;
        row.cat_collision_rate /= n;
    }
    LeakReport {
        rows,
        honest_flagged_fixed: honest_flagged as f64 / people.max(1) as f64,
        leak_db_instances: db_size,
    }
}

#[derive(Serialize)]
pub struct AssessmentReport {
    pub scenarios: Vec<Scenario>,
    pub items: ItemQuality,
    pub leak: LeakReport,
    pub templates_total: usize,
}

pub fn run(seed: u64) -> AssessmentReport {
    AssessmentReport {
        scenarios: vec![
            scenario(seed, "честная заявка, точная калибровка", false, false),
            scenario(
                seed + 1,
                "заявка с ошибками (20 % завышают, 10 % занижают)",
                false,
                true,
            ),
            scenario(
                seed + 2,
                "ошибка калибровки заданий (a ×0.8…1.2, b ±0.3)",
                true,
                false,
            ),
        ],
        items: item_quality(seed + 3),
        leak: leak(seed + 4),
        templates_total: assessment::bank().templates().len(),
    }
}

pub fn markdown(r: &AssessmentReport) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "## Тестирование: точность, стабильность, дискриминативность, устойчивость к утечкам\n"
    );
    let _ = writeln!(
        s,
        "Каждая сессия — настоящий движок `assessment` (адаптивный выбор из {} шаблонов, EAP-оценка θ, правило остановки). \
         Синтетическими остаются только ответы: вероятность верного ответа по модели 3PL при истинной способности θ ~ N(0, 1). \
         По {} человек на специализацию; процедура как в API и по той же политике грейда (`GradePolicy`): тест на заявленный грейд, при провале — сразу на грейд ниже, вплоть до Junior. \
         Пересдача — вторая независимая процедура с другими заданиями.\n",
        r.templates_total, PER_SPEC
    );
    let _ = writeln!(
        s,
        "| Сценарий | Решение «выше/ниже порога» верно | Итоговый грейд = истинный | Отклонение ≤ 1 грейда | Совпадение при пересдаче | κ Коэна | RMSE θ | Смещение θ | Заданий в тесте |\n|---|---|---|---|---|---|---|---|---|"
    );
    for x in &r.scenarios {
        let _ = writeln!(
            s,
            "| {} | {:.1} % | {:.1} % | {:.1} % | {:.1} % | {:.2} | {:.2} | {:+.2} | {:.1} |",
            x.name,
            x.decision_accuracy * 100.0,
            x.final_grade_accuracy * 100.0,
            x.within_one_grade * 100.0,
            x.retest_agreement * 100.0,
            x.retest_kappa,
            x.theta_rmse,
            x.theta_bias,
            x.mean_items
        );
    }
    if let Some(x) = r.scenarios.first() {
        let _ = writeln!(
            s,
            "\nМатрица ошибок (честная заявка): строки — истинный грейд, столбцы — присвоенный.\n\n| Истина \\ Присвоено | {} |\n|---|---|---|---|---|",
            GRADE_LABELS.join(" | ")
        );
        for (i, row) in x.confusion.iter().enumerate() {
            let _ = writeln!(
                s,
                "| {} | {} |",
                GRADE_LABELS[i],
                row.iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
    let q = &r.items;
    let _ = writeln!(
        s,
        "\n### Дискриминативность и экспозиция\n\nТочечно-бисериальная корреляция верного ответа с истинной θ: в среднем {:.2}, \
         у {:.0} % шаблонов ≥ 0.2. Экспозиция шаблона (доля сессий, где он показан): медиана {:.0} %, максимум {:.0} %.\n\n\
         Самые слабые шаблоны (кандидаты на перекалибровку или замену):\n",
        q.mean_point_biserial,
        q.share_above_02 * 100.0,
        q.median_exposure * 100.0,
        q.max_exposure * 100.0
    );
    for t in q.templates.iter().take(5) {
        let _ = writeln!(
            s,
            "- `{}`: r = {:.2} (показан {} раз)",
            t.template_id, t.point_biserial, t.shown
        );
    }
    let l = &r.leak;
    let _ = writeln!(
        s,
        "\n### Утечка заданий\n\nСлабые кандидаты (истинная θ от −0.9 до −0.1, т. е. ниже порога Middle) сдают тест на Middle. \
         **Фиксированный тест**: одна публичная форма из {FIXED_FORM_LEN} самых информативных заданий, известна доля ответов. \
         **Наш тест**: база утечки — все {} экземпляров, показанных в {} предыдущих сессиях на специализацию; кандидат отвечает верно, \
         если ему выпал экземпляр из базы.\n\n| Утекло | Фиксированный тест: прошли | Фиксированный тест: флаг person-fit | Наш тест: прошли | Наш тест: доля заданий из базы утечки |\n|---|---|---|---|---|",
        l.leak_db_instances, LEAK_DB_SESSIONS
    );
    for row in &l.rows {
        let _ = writeln!(
            s,
            "| {:.0} % | {:.1} % | {:.1} % | {:.1} % | {:.2} % |",
            row.leak_share * 100.0,
            row.fixed_form_pass_rate * 100.0,
            row.fixed_form_flagged * 100.0,
            row.cat_pass_rate * 100.0,
            row.cat_collision_rate * 100.0
        );
    }
    let _ = writeln!(
        s,
        "\nДоля ложных флагов person-fit у честных кандидатов фиксированного теста: {:.1} %.",
        l.honest_flagged_fixed * 100.0
    );
    s
}
