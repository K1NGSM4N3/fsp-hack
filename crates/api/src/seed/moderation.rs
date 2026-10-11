//! What the moderator's cabinet shows on a fresh demo: the moderator account, tests waiting for a
//! decision and anti-fake items. Draws come from their own RNG stream, so every other demo row
//! stays exactly as before.

use catalog::Category;
use chrono::{DateTime, Duration, Utc};
use matching::trust::{MarketBand, salary_flags};
use rand::{Rng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    crypto::{Crypto, normalize_email},
    error::ApiResult,
};

pub const DEMO_MODERATOR_EMAIL: &str = "moderator@demo.fsp";
/// The candidate whose test opens the queue: «требует подтверждения», video, tab leaves.
pub const DEMO_DISPUTED_EMAIL: &str = "review@demo.fsp";
pub const MODERATION_SEED: u64 = 7707;
/// «Логистика Плюс Цифра»: its vacancies get flags and complaints.
const FLAGGED_COMPANY: usize = 4;
const OTHER_COMPANY: usize = 1;

/// A candidate's assigning session, as the main seed stored it.
pub struct Reviewable {
    pub candidate: Uuid,
    pub session: Uuid,
    pub finished: DateTime<Utc>,
    /// The seed already wrote random remarks for this candidate.
    pub remarks: bool,
}

#[derive(Default)]
pub struct ModerationReport {
    /// Person-fit flag with tab leaves or pastes.
    pub flagged_with_events: usize,
    /// Person-fit flag on a test without proctoring data.
    pub flagged_without_proctoring: usize,
    /// Remarks without the person-fit flag.
    pub remarks_only: usize,
    pub flagged_vacancies: usize,
    pub complaints: usize,
}

pub async fn seed_moderator(
    tx: &mut PgConnection,
    crypto: &Crypto,
    hash: &str,
    now: DateTime<Utc>,
) -> ApiResult<Uuid> {
    let id = Uuid::now_v7();
    let email = normalize_email(DEMO_MODERATOR_EMAIL);
    db::seed::seed_verified_user(
        &mut *tx,
        id,
        db::types::UserRole::Admin,
        &crypto.encrypt_str(&email)?,
        &crypto.blind_index(&email),
        hash,
        now - Duration::days(60),
    )
    .await?;
    Ok(id)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Hard items right, easy ones wrong, and the candidate left the tab on those hard items.
    FlaggedWithEvents,
    /// The same pattern on a test taken with proctoring off.
    FlaggedWithoutProctoring,
    /// A couple of window switches on an ordinary pattern.
    RemarksOnly,
}

struct Answer {
    seq: i32,
    params: assessment::ItemParams,
    correct: bool,
    shown: DateTime<Utc>,
    answered: DateTime<Utc>,
}

/// Flips the easiest right answers to wrong and the hardest wrong ones to right until `l_z` at
/// the stored θ falls below the flag, keeping the number of right answers about the same.
fn aberrant(theta: f64, answers: &mut [Answer]) -> f64 {
    let lz = |a: &[Answer]| {
        assessment::irt::person_fit_lz(theta, a.iter().map(|x| (&x.params, x.correct)))
    };
    let mut order: Vec<usize> = (0..answers.len()).collect();
    order.sort_by(|&a, &b| answers[a].params.b.total_cmp(&answers[b].params.b));
    let mut value = lz(answers);
    for k in 0..answers.len() / 2 {
        if value < -2.3 {
            break;
        }
        let (easy, hard) = (order[k], order[answers.len() - 1 - k]);
        answers[easy].correct = false;
        answers[hard].correct = true;
        value = lz(answers);
    }
    value
}

fn moment(rng: &mut ChaCha8Rng, a: &Answer, from: f64, to: f64) -> DateTime<Utc> {
    let span = (a.answered - a.shown).num_milliseconds().max(1) as f64;
    a.shown + Duration::milliseconds((span * rng.random_range(from..to)) as i64)
}

async fn reshape(
    tx: &mut PgConnection,
    rng: &mut ChaCha8Rng,
    r: &Reviewable,
    shape: Shape,
) -> ApiResult<()> {
    let bank = assessment::bank();
    let rows = db::moderation::responses(&mut *tx, r.session).await?;
    let mut answers: Vec<Answer> = rows
        .iter()
        .filter_map(|row| {
            let level = u8::try_from(row.level).ok()?;
            let params = bank.template(&row.template_id)?.level(level)?.params;
            let answered = row.answered_at;
            Some(Answer {
                seq: row.seq,
                params,
                correct: row.correct,
                shown: answered - Duration::milliseconds(i64::from(row.elapsed_ms)),
                answered,
            })
        })
        .collect();
    let mut state: assessment::SessionState =
        serde_json::from_value(db::seed::seed_session_state(&mut *tx, r.session).await?)?;
    let theta = state.summary.theta;
    let before: Vec<bool> = answers.iter().map(|a| a.correct).collect();
    let flagged = shape != Shape::RemarksOnly;
    let lz = if flagged {
        aberrant(theta, &mut answers)
    } else {
        assessment::irt::person_fit_lz(theta, answers.iter().map(|a| (&a.params, a.correct)))
    };
    for (a, was) in answers.iter().zip(&before) {
        if a.correct != *was {
            db::seed::seed_response_correct(&mut *tx, r.session, a.seq, a.correct).await?;
            if let Some(item) = usize::try_from(a.seq)
                .ok()
                .and_then(|i| state.administered.get_mut(i))
            {
                item.correct = a.correct;
            }
        }
    }
    db::seed::seed_review_flag(
        &mut *tx,
        r.session,
        lz,
        flagged,
        &serde_json::to_value(&state)?,
    )
    .await?;

    let event = |kind, at, item_seq: i32, duration_ms| db::proctoring::Event {
        kind,
        occurred_at: at,
        item_seq: i16::try_from(item_seq).ok(),
        duration_ms,
    };
    match shape {
        Shape::FlaggedWithoutProctoring => {
            db::seed::seed_drop_proctoring(&mut *tx, r.session).await?;
        }
        Shape::FlaggedWithEvents => {
            let mut hard: Vec<&Answer> = answers
                .iter()
                .zip(&before)
                .filter(|(a, was)| a.correct && !**was)
                .map(|(a, _)| a)
                .collect();
            if hard.is_empty() {
                hard = answers.iter().filter(|a| a.correct).take(2).collect();
            }
            for a in hard.iter().take(3) {
                let e = event(
                    db::types::IntegrityEvent::TabHidden,
                    moment(rng, a, 0.15, 0.45),
                    a.seq,
                    Some(rng.random_range(12_000..48_000)),
                );
                db::proctoring::insert_event(&mut *tx, r.session, &e).await?;
            }
            if let Some(a) = hard.first() {
                let e = event(
                    db::types::IntegrityEvent::Paste,
                    moment(rng, a, 0.7, 0.95),
                    a.seq,
                    None,
                );
                db::proctoring::insert_event(&mut *tx, r.session, &e).await?;
            }
        }
        Shape::RemarksOnly => {
            let mut picks: Vec<&Answer> = answers.iter().collect();
            picks.shuffle(rng);
            for a in picks.into_iter().take(2) {
                let e = event(
                    db::types::IntegrityEvent::WindowBlur,
                    moment(rng, a, 0.2, 0.8),
                    a.seq,
                    Some(rng.random_range(3_000..15_000)),
                );
                db::proctoring::insert_event(&mut *tx, r.session, &e).await?;
            }
        }
    }
    Ok(())
}

/// Seven tests for the proctoring queue: the five most recent assigning sessions and the two
/// oldest (their video is past retention), among candidates without random remarks. The most
/// recent flagged one logs in as [`DEMO_DISPUTED_EMAIL`].
pub async fn seed_reviews(
    tx: &mut PgConnection,
    crypto: &Crypto,
    rng: &mut ChaCha8Rng,
    reviewables: &[Reviewable],
    report: &mut ModerationReport,
) -> ApiResult<Vec<Uuid>> {
    let mut quiet: Vec<&Reviewable> = reviewables.iter().filter(|r| !r.remarks).collect();
    quiet.sort_by_key(|r| std::cmp::Reverse(r.finished));
    if quiet.len() < 7 {
        return Ok(Vec::new());
    }
    let oldest = quiet.len() - 2;
    let picks = [
        (quiet[0], Shape::FlaggedWithEvents),
        (quiet[1], Shape::FlaggedWithEvents),
        (quiet[2], Shape::FlaggedWithoutProctoring),
        (quiet[3], Shape::RemarksOnly),
        (quiet[4], Shape::FlaggedWithoutProctoring),
        (quiet[oldest], Shape::FlaggedWithEvents),
        (quiet[oldest + 1], Shape::RemarksOnly),
    ];
    for (r, shape) in picks {
        reshape(&mut *tx, rng, r, shape).await?;
        match shape {
            Shape::FlaggedWithEvents => report.flagged_with_events += 1,
            Shape::FlaggedWithoutProctoring => report.flagged_without_proctoring += 1,
            Shape::RemarksOnly => report.remarks_only += 1,
        }
    }
    let email = normalize_email(DEMO_DISPUTED_EMAIL);
    db::seed::seed_login_email(
        &mut *tx,
        quiet[0].candidate,
        &crypto.encrypt_str(&email)?,
        &crypto.blind_index(&email),
    )
    .await?;
    Ok(picks.iter().map(|(r, _)| r.candidate).collect())
}

/// Two flagged vacancies of one company — a range wider than three times its floor and a ceiling
/// far above the category market — and three open complaints from candidates: about one of those
/// vacancies, about the company, and about another company's vacancy.
pub async fn seed_trust(
    tx: &mut PgConnection,
    rng: &mut ChaCha8Rng,
    companies: &[Uuid],
    reporters: &[Uuid],
    now: DateTime<Utc>,
    report: &mut ModerationReport,
) -> ApiResult<()> {
    let (Some(&flagged), Some(&other)) =
        (companies.get(FLAGGED_COMPANY), companies.get(OTHER_COMPANY))
    else {
        return Ok(());
    };
    if reporters.len() < 3 {
        return Ok(());
    }
    let vacancies = db::seed::seed_company_vacancies(&mut *tx, flagged).await?;
    let mut targets: Vec<Uuid> = Vec::new();
    for (i, v) in vacancies.iter().take(2).enumerate() {
        let category = Category::new(v.specialization.parse()?, v.grade.parse()?);
        let salary = if i == 0 {
            (v.salary_min / 3 / 10_000 * 10_000, v.salary_max)
        } else {
            let band = db::trust::expectation_band(&mut *tx, &v.specialization, &v.grade).await?;
            let p90 = band.p90.unwrap_or(v.salary_max);
            let max = (p90 * 19 / 10 + 9_999) / 10_000 * 10_000;
            (max * 6 / 10, max)
        };
        let band = db::trust::expectation_band(&mut *tx, &v.specialization, &v.grade).await?;
        let market = match (band.median, band.p90) {
            (Some(median), Some(p90)) => Some(MarketBand {
                samples: u32::try_from(band.samples).unwrap_or(u32::MAX),
                median,
                p90,
            }),
            _ => None,
        };
        let flags = salary_flags(category, salary.0, salary.1, market);
        if flags.is_empty() {
            continue;
        }
        let at = now - Duration::hours(rng.random_range(30..120));
        db::seed::seed_vacancy_flags(&mut *tx, v.id, salary, &serde_json::to_value(&flags)?, at)
            .await?;
        report.flagged_vacancies += 1;
        targets.push(v.id);
    }
    let elsewhere = db::seed::seed_company_vacancies(&mut *tx, other).await?;
    let complaints: [(&str, Option<Uuid>, &str, &str); 3] = [
        (
            "vacancy",
            targets.last().copied(),
            "fake_vacancy",
            "На созвоне сказали, что такой зарплаты нет и не было, предложили вдвое меньше.",
        ),
        (
            "company",
            Some(flagged),
            "spam",
            "Третий раз за неделю пишут про одну и ту же позицию, хотя я отказался.",
        ),
        (
            "vacancy",
            elsewhere.last().map(|v| v.id),
            "salary_mismatch",
            "В вакансии одна вилка, а в письме после отклика — другая.",
        ),
    ];
    for (i, (kind, target, reason, comment)) in complaints.into_iter().enumerate() {
        let Some(target) = target else {
            continue;
        };
        db::seed::seed_report(
            &mut *tx,
            &db::seed::SeedReport {
                id: Uuid::now_v7(),
                reporter_user_id: reporters[i],
                target_kind: kind,
                target_id: target,
                reason,
                comment,
                created_at: now - Duration::hours(rng.random_range(2..72)),
            },
        )
        .await?;
        report.complaints += 1;
    }
    Ok(())
}
