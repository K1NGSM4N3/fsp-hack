//! Company trust level and the weekly invitation quota it allows.
//!
//! The rule in plain words: a new company starts with 10 invitations a week. A confirmed
//! corporate domain, a month on the platform and three invitations accepted by candidates add 10
//! each; bonuses stack up to +20, so any two of the three reach the maximum of 30 and a company
//! with a public mailbox is not stuck below it. Complaints, declines "because of the salary or the
//! company", frequent withdrawals, ignored invitations, inflated salary ranges and copied vacancies
//! take points away, but the quota never drops below 5: an honest company that got one unfair
//! complaint can still work. Every step is a reason the company sees.

use serde::{Deserialize, Serialize};

use crate::ru::plural;

pub const BASE_QUOTA: i16 = 10;
pub const MAX_QUOTA: i16 = 30;
pub const MIN_QUOTA: i16 = 5;
const STEP: i16 = 10;
pub const MAX_BONUS: i16 = MAX_QUOTA - BASE_QUOTA;

/// Account age after which the company is no longer "new".
pub const MATURE_DAYS: i64 = 30;
/// Accepted invitations (last 180 days) that show candidates trust the company; half of the
/// bonus from the first one.
pub const ACCEPTED_FOR_BONUS: u16 = 3;

const COMPLAINT_PENALTY: i16 = 10;
const COMPLAINT_CAP: i16 = 30;
/// Declines "зарплата не соответствует" or "компания" among at least this many declines...
pub const BAD_DECLINE_MIN: u16 = 5;
/// ...reaching this share mean the invitation promised something the company is not.
pub const BAD_DECLINE_SHARE: f32 = 0.4;
const BAD_DECLINE_PENALTY: i16 = 10;
pub const WITHDRAW_MIN: u16 = 5;
/// Withdrawing a third of invitations looks like mass-inviting first and choosing later.
pub const WITHDRAW_SHARE: f32 = 0.3;
const WITHDRAW_PENALTY: i16 = 5;
pub const NO_ANSWER_MIN: u16 = 10;
/// Candidates answer most real invitations; nine of ten left to expire look like spam.
pub const NO_ANSWER_SHARE: f32 = 0.1;
const NO_ANSWER_PENALTY: i16 = 5;
const SALARY_FLAG_PENALTY: i16 = 5;
const SALARY_FLAG_CAP: i16 = 10;
const DUPLICATE_PENALTY: i16 = 5;
const COPY_PENALTY: i16 = 20;

const SECONDS_PER_DAY: i64 = 86_400;

/// Counters from the database. Windows: 90 days for behaviour, 180 days for accepted invitations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrustSignals {
    pub created_unix: i64,
    pub domain_verified: bool,
    /// Distinct candidates who complained about the company, its vacancies or invitations.
    pub complaints: u16,
    pub declines: u16,
    /// Declined with reason `salary` or `company`.
    pub bad_declines: u16,
    pub invitations: u16,
    pub withdrawn: u16,
    /// Invitations no longer waiting for an answer: accepted, declined or expired.
    pub closed: u16,
    /// Accepted or declined.
    pub answered: u16,
    pub accepted: u16,
    /// Vacancies and invitations whose range was flagged.
    pub salary_flags: u16,
    pub own_duplicates: u16,
    pub copies: u16,
    /// A moderator froze the company's invitations until this moment (unix seconds).
    pub frozen_until: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// A moderator froze invitations: the quota is 0 until the freeze ends.
    Frozen,
    /// Negative signals took the quota down to the base or below.
    Low,
    /// Under a month on the platform, nothing negative.
    New,
    Standard,
    /// Full quota.
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum TrustReasonKind {
    Domain,
    AccountAge,
    Accepted,
    Complaints,
    BadDeclines,
    Withdrawals,
    NoAnswers,
    SalaryFlags,
    OwnDuplicates,
    Copies,
    /// Invitations frozen by a moderator.
    Frozen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TrustReason {
    pub kind: TrustReasonKind,
    /// Change of the weekly quota.
    pub points: i16,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TrustAssessment {
    pub level: TrustLevel,
    /// Invitations per 7 days: `base + bonus + penalty`, kept within 5..=30.
    pub quota: i16,
    pub base: i16,
    /// Sum of positive reasons, capped at +20.
    pub bonus: i16,
    /// Sum of negative reasons (≤ 0).
    pub penalty: i16,
    pub reasons: Vec<TrustReason>,
    /// What raises the quota, most effective first.
    pub advice: Vec<String>,
}

impl TrustAssessment {
    pub fn has_negative(&self) -> bool {
        self.reasons.iter().any(|r| r.points < 0)
    }
}

fn share(part: u16, total: u16) -> f32 {
    if total == 0 {
        0.0
    } else {
        f32::from(part) / f32::from(total)
    }
}

fn percent(x: f32) -> u32 {
    (x * 100.0).round() as u32
}

fn n(x: u16) -> usize {
    usize::from(x)
}

/// `DD.MM.YYYY` in UTC, like the dates in the API's problem details.
fn ru_date(unix: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(unix).map_or_else(
        |_| String::new(),
        |t| format!("{:02}.{:02}.{}", t.day(), u8::from(t.month()), t.year()),
    )
}

pub fn assess(s: &TrustSignals, now_unix: i64) -> TrustAssessment {
    let age_days = ((now_unix - s.created_unix) / SECONDS_PER_DAY).max(0);
    let mut reasons = Vec::new();
    let mut advice = Vec::new();
    let mut push = |kind, points, text: String| reasons.push(TrustReason { kind, points, text });

    if s.domain_verified {
        push(
            TrustReasonKind::Domain,
            STEP,
            "Домен подтверждён: почта для входа и сайт компании на одном домене".into(),
        );
    }
    if age_days >= MATURE_DAYS {
        push(
            TrustReasonKind::AccountAge,
            STEP,
            format!(
                "На платформе {age_days} {}",
                plural(age_days as usize, ["день", "дня", "дней"])
            ),
        );
    }
    if s.accepted > 0 {
        let points = if s.accepted >= ACCEPTED_FOR_BONUS {
            STEP
        } else {
            STEP / 2
        };
        push(
            TrustReasonKind::Accepted,
            points,
            format!(
                "Кандидаты приняли {} {} за полгода",
                s.accepted,
                plural(n(s.accepted), ["приглашение", "приглашения", "приглашений"])
            ),
        );
    }
    if s.complaints > 0 {
        push(
            TrustReasonKind::Complaints,
            -(COMPLAINT_PENALTY * s.complaints as i16).min(COMPLAINT_CAP),
            format!(
                "{} {} от кандидатов за 90 дней",
                s.complaints,
                plural(n(s.complaints), ["жалоба", "жалобы", "жалоб"])
            ),
        );
    }
    let bad = share(s.bad_declines, s.declines);
    if s.declines >= BAD_DECLINE_MIN && bad >= BAD_DECLINE_SHARE {
        push(
            TrustReasonKind::BadDeclines,
            -BAD_DECLINE_PENALTY,
            format!(
                "{} % отказов — «зарплата не соответствует» или «компания»",
                percent(bad)
            ),
        );
    }
    let withdrawn = share(s.withdrawn, s.invitations);
    if s.invitations >= WITHDRAW_MIN && withdrawn >= WITHDRAW_SHARE {
        push(
            TrustReasonKind::Withdrawals,
            -WITHDRAW_PENALTY,
            format!("Отозвано {} % приглашений за 90 дней", percent(withdrawn)),
        );
    }
    let answered = share(s.answered, s.closed);
    if s.closed >= NO_ANSWER_MIN && answered <= NO_ANSWER_SHARE {
        push(
            TrustReasonKind::NoAnswers,
            -NO_ANSWER_PENALTY,
            format!(
                "Кандидаты ответили только на {} % приглашений",
                percent(answered)
            ),
        );
    }
    if s.salary_flags > 0 {
        push(
            TrustReasonKind::SalaryFlags,
            -(SALARY_FLAG_PENALTY * s.salary_flags as i16).min(SALARY_FLAG_CAP),
            format!(
                "{} {} с вилкой заметно выше рынка категории или слишком широкой",
                s.salary_flags,
                plural(
                    n(s.salary_flags),
                    [
                        "вакансия или приглашение",
                        "вакансии или приглашения",
                        "вакансий или приглашений"
                    ]
                )
            ),
        );
    }
    if s.own_duplicates > 0 {
        push(
            TrustReasonKind::OwnDuplicates,
            -DUPLICATE_PENALTY,
            format!(
                "{} {} почти повторяет другую вашу вакансию",
                s.own_duplicates,
                plural(n(s.own_duplicates), ["вакансия", "вакансии", "вакансий"])
            ),
        );
    }
    if s.copies > 0 {
        push(
            TrustReasonKind::Copies,
            -COPY_PENALTY,
            format!(
                "{} {} почти дословно совпадает с вакансией другой компании",
                s.copies,
                plural(n(s.copies), ["вакансия", "вакансии", "вакансий"])
            ),
        );
    }

    let bonus = reasons
        .iter()
        .map(|r| r.points.max(0))
        .sum::<i16>()
        .min(MAX_BONUS);
    let mut penalty: i16 = reasons.iter().map(|r| r.points.min(0)).sum();
    let mut quota = (BASE_QUOTA + bonus + penalty).clamp(MIN_QUOTA, MAX_QUOTA);
    let negative = penalty < 0;
    let mut level = if negative && quota <= BASE_QUOTA {
        TrustLevel::Low
    } else if quota >= MAX_QUOTA {
        TrustLevel::High
    } else if age_days < MATURE_DAYS && !negative {
        TrustLevel::New
    } else {
        TrustLevel::Standard
    };

    if s.copies > 0 {
        advice.push("Перепишите скопированный текст своими словами: отметка о совпадении перестаёт учитываться через 90 дней после публикации.".into());
    }
    if s.complaints > 0 {
        advice.push(
            "Жалобы разбирает модератор; необоснованные снимаются и перестают снижать квоту."
                .into(),
        );
    }
    if s.declines >= BAD_DECLINE_MIN && bad >= BAD_DECLINE_SHARE {
        advice.push("Сверьте вилку с медианными ожиданиями категории и дополните профиль компании: кандидаты отказываются, когда условия расходятся с ожиданиями.".into());
    }
    if s.salary_flags > 0 {
        advice.push("Указывайте вилку, которую действительно готовы платить: подсказка о рынке категории появляется прямо в форме.".into());
    }
    if s.own_duplicates > 0 {
        advice.push("Не публикуйте копию вакансии — обновите существующую, тогда отклики соберутся в одном месте.".into());
    }
    if s.invitations >= WITHDRAW_MIN && withdrawn >= WITHDRAW_SHARE {
        advice.push("Приглашайте тех, в ком уверены, — отзыв приглашения кандидаты видят.".into());
    }
    if s.closed >= NO_ANSWER_MIN && answered <= NO_ANSWER_SHARE {
        advice.push("Напишите в приглашении, чем кандидат интересен именно вам: на точечные предложения отвечают чаще.".into());
    }
    let growth_from = advice.len();
    if !s.domain_verified {
        advice.push(format!(
            "Подтвердите корпоративный домен — квота вырастет на {STEP}."
        ));
    }
    if age_days < MATURE_DAYS {
        let left = (MATURE_DAYS - age_days) as usize;
        advice.push(format!(
            "Через {left} {} на платформе квота вырастет на {STEP}.",
            plural(left, ["день", "дня", "дней"])
        ));
    }
    if s.accepted < ACCEPTED_FOR_BONUS {
        let left = n(ACCEPTED_FOR_BONUS - s.accepted);
        advice.push(format!(
            "Когда кандидаты примут ещё {left} {}, квота вырастет на {}.",
            plural(left, ["приглашение", "приглашения", "приглашений"]),
            if s.accepted == 0 { STEP } else { STEP / 2 }
        ));
    }
    if bonus >= MAX_BONUS || quota >= MAX_QUOTA {
        advice.truncate(growth_from);
    }
    // The freeze is a moderator's decision, not a signal: it takes the whole quota, below the
    // floor of 5 that protects a company from heuristics.
    if let Some(until) = s.frozen_until.filter(|&u| u > now_unix) {
        reasons.push(TrustReason {
            kind: TrustReasonKind::Frozen,
            points: -quota,
            text: format!("Модератор приостановил приглашения до {}", ru_date(until)),
        });
        penalty -= quota;
        quota = 0;
        level = TrustLevel::Frozen;
        advice.insert(
            0,
            format!(
                "Приглашения снова станут доступны {}. Решение и комментарий модератора — в разделе «Решения модератора».",
                ru_date(until)
            ),
        );
    }

    TrustAssessment {
        level,
        quota,
        base: BASE_QUOTA,
        bonus,
        penalty,
        reasons,
        advice,
    }
}

/// The quota rule, line by line, as the employer reads it.
pub fn quota_rule() -> Vec<String> {
    vec![
        format!("Новая компания начинает с {BASE_QUOTA} приглашений в неделю."),
        format!(
            "+{STEP} за подтверждённый домен, +{STEP} после {MATURE_DAYS} дней на платформе, +{STEP} когда кандидаты примут {ACCEPTED_FOR_BONUS} приглашения (+{} — после первого). Бонусы складываются до +{MAX_BONUS}: любых двух шагов хватает до максимума — {MAX_QUOTA}.",
            STEP / 2
        ),
        format!(
            "−{COMPLAINT_PENALTY} за каждую жалобу кандидата за 90 дней (не больше −{COMPLAINT_CAP}); −{BAD_DECLINE_PENALTY}, если от {} % отказов — из-за зарплаты или компании; −{SALARY_FLAG_PENALTY} за вилку заметно выше рынка (не больше −{SALARY_FLAG_CAP}); −{COPY_PENALTY} за вакансию, скопированную у другой компании.",
            percent(BAD_DECLINE_SHARE)
        ),
        format!(
            "−{WITHDRAW_PENALTY}, если отозвано от {} % приглашений; −{NO_ANSWER_PENALTY}, если кандидаты отвечают на {} % и реже; −{DUPLICATE_PENALTY} за копии собственных вакансий.",
            percent(WITHDRAW_SHARE),
            percent(NO_ANSWER_SHARE)
        ),
        format!("Квота не опускается ниже {MIN_QUOTA}: одна ошибка не останавливает работу."),
        "Модератор может приостановить приглашения на срок до 90 дней: на это время квота — 0, причина и срок видны здесь же.".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_790_000_000;
    const DAY: i64 = SECONDS_PER_DAY;

    fn company(age_days: i64) -> TrustSignals {
        TrustSignals {
            created_unix: NOW - age_days * DAY,
            ..TrustSignals::default()
        }
    }

    #[test]
    fn new_company_starts_at_ten() {
        let a = assess(&company(2), NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::New, 10));
        assert!(a.reasons.is_empty());
        assert_eq!(a.advice.len(), 3);
        assert!(a.advice[1].contains("Через 28 дней"));
    }

    #[test]
    fn quota_grows_to_thirty_with_good_behaviour() {
        let mut s = company(2);
        s.domain_verified = true;
        assert_eq!(assess(&s, NOW).quota, 20);
        s.accepted = 1;
        assert_eq!(assess(&s, NOW).quota, 25);
        s.accepted = 3;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::High, 30));
        assert!(a.advice.is_empty());

        let mut old = company(120);
        assert_eq!(
            (assess(&old, NOW).level, assess(&old, NOW).quota),
            (TrustLevel::Standard, 20)
        );
        old.accepted = 4;
        assert_eq!(assess(&old, NOW).quota, 30);
        old.domain_verified = true;
        assert_eq!(assess(&old, NOW).quota, 30);
    }

    #[test]
    fn complaints_lower_the_quota_but_not_below_five() {
        let mut s = company(120);
        s.domain_verified = true;
        s.complaints = 1;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Standard, 20));
        s.complaints = 2;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Low, 10));
        s.complaints = 7;
        let a = assess(&s, NOW);
        assert_eq!(a.quota, 5);
        let complaints = a
            .reasons
            .iter()
            .find(|r| r.kind == TrustReasonKind::Complaints)
            .unwrap();
        assert_eq!(complaints.points, -30);
        assert_eq!(complaints.text, "7 жалоб от кандидатов за 90 дней");
    }

    #[test]
    fn decline_reasons_need_enough_declines() {
        let mut s = company(120);
        s.declines = 4;
        s.bad_declines = 4;
        assert_eq!(assess(&s, NOW).quota, 20);
        s.declines = 5;
        s.bad_declines = 2;
        let a = assess(&s, NOW);
        assert_eq!(a.quota, 10);
        assert!(a.reasons.iter().any(|r| r.text.starts_with("40 % отказов")));
        s.bad_declines = 1;
        assert_eq!(assess(&s, NOW).quota, 20);
    }

    #[test]
    fn withdrawals_and_silence() {
        let mut s = company(120);
        s.invitations = 10;
        s.withdrawn = 3;
        assert_eq!(assess(&s, NOW).quota, 15);
        s.withdrawn = 2;
        assert_eq!(assess(&s, NOW).quota, 20);
        s.closed = 10;
        s.answered = 1;
        assert_eq!(assess(&s, NOW).quota, 15);
        s.answered = 2;
        assert_eq!(assess(&s, NOW).quota, 20);
    }

    #[test]
    fn salary_flags_duplicates_and_copies() {
        let mut s = company(120);
        s.domain_verified = true;
        s.accepted = 3;
        s.salary_flags = 1;
        assert_eq!(assess(&s, NOW).quota, 25);
        s.salary_flags = 5;
        assert_eq!(assess(&s, NOW).quota, 20);
        s.own_duplicates = 2;
        assert_eq!(assess(&s, NOW).quota, 15);
        s.copies = 1;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Low, 5));
        assert!(a.advice[0].starts_with("Перепишите"));
    }

    #[test]
    fn every_point_is_explained() {
        let mut s = company(45);
        s.domain_verified = true;
        s.accepted = 2;
        s.complaints = 1;
        s.salary_flags = 1;
        let a = assess(&s, NOW);
        assert_eq!((a.bonus, a.penalty), (20, -15));
        assert_eq!(a.quota, a.base + a.bonus + a.penalty);
        assert_eq!(a.quota, 15);
        assert!(a.reasons.iter().all(|r| !r.text.is_empty()));
    }

    #[test]
    fn freeze_takes_the_whole_quota_until_it_ends() {
        let mut s = company(120);
        s.domain_verified = true;
        s.frozen_until = Some(NOW + 7 * DAY);
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Frozen, 0));
        let freeze = a
            .reasons
            .iter()
            .find(|r| r.kind == TrustReasonKind::Frozen)
            .unwrap();
        assert_eq!(freeze.points, -30);
        assert!(
            freeze
                .text
                .starts_with("Модератор приостановил приглашения до ")
        );
        assert!(a.has_negative());
        assert!(a.advice[0].starts_with("Приглашения снова станут доступны"));

        s.complaints = 7;
        assert_eq!(assess(&s, NOW).quota, 0, "below the floor of 5");

        s.frozen_until = Some(NOW);
        let after = assess(&s, NOW);
        assert_eq!(after.quota, 5);
        assert!(
            after
                .reasons
                .iter()
                .all(|r| r.kind != TrustReasonKind::Frozen)
        );
    }

    fn kinds(a: &TrustAssessment) -> Vec<TrustReasonKind> {
        a.reasons.iter().map(|r| r.kind).collect()
    }

    #[test]
    fn a_new_company_with_a_flagged_range_and_an_own_copy_drops_to_the_floor() {
        let mut s = company(0);
        s.salary_flags = 1;
        s.own_duplicates = 1;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Low, MIN_QUOTA));
        assert_eq!(
            kinds(&a),
            [TrustReasonKind::SalaryFlags, TrustReasonKind::OwnDuplicates]
        );
        assert_eq!(a.penalty, -10);
    }

    #[test]
    fn complaints_outweigh_a_verified_domain_down_to_the_floor() {
        let mut s = company(0);
        s.domain_verified = true;
        s.complaints = 2;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::Low, MIN_QUOTA));
        assert_eq!((a.bonus, a.penalty), (10, -20));
        assert_eq!(
            kinds(&a),
            [TrustReasonKind::Domain, TrustReasonKind::Complaints]
        );
    }

    #[test]
    fn the_first_accepted_invitation_gives_a_new_company_half_a_step() {
        let mut s = company(0);
        s.accepted = 1;
        let a = assess(&s, NOW);
        assert_eq!((a.level, a.quota), (TrustLevel::New, 15));
        assert_eq!(a.reasons[0].points, 5);
        assert_eq!(
            a.reasons[0].text,
            "Кандидаты приняли 1 приглашение за полгода"
        );
        assert!(
            a.advice
                .last()
                .unwrap()
                .starts_with("Когда кандидаты примут ещё 2 приглашения, квота вырастет на 5")
        );
    }

    #[test]
    fn rule_mentions_the_bounds() {
        let rule = quota_rule().join(" ");
        assert!(rule.contains("10 приглашений") && rule.contains("максимума — 30"));
        assert!(rule.contains("ниже 5"));
    }
}
