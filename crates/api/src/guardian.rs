//! Consent of a minor candidate's legal representative (architecture.md §9.5). After the minor
//! confirms their own email, the representative gets a letter with two links of one family: a
//! one-time answer link (7 days) and a revocation link. Both open the public page
//! `/guardian/confirm`. Data of the representative is erased 30 days after the 18th birthday.

use std::time::Duration;

use chrono::{DateTime, Utc};
use db::types::GuardianDecision;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    age::GUARDIAN_RETENTION_DAYS,
    auth::routes::valid_email,
    crypto::{Crypto, new_token, normalize_email},
    error::{ApiError, ApiResult, Validator},
    state::AppState,
};

pub const CONFIRM_TTL_DAYS: i64 = 7;
const SWEEP_EVERY: Duration = Duration::from_secs(6 * 3600);
const SUBJECT: &str = "Согласие на участие в «ФСП Таланты»";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GuardianState {
    /// The minor has not confirmed their own email yet, so no letter went out.
    NotSent,
    /// The letter is out and its answer link still works.
    Pending,
    /// The answer link expired without an answer; the minor can send the letter again.
    Expired,
    Granted,
    Declined,
    /// Granted, then revoked by the representative.
    Revoked,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GuardianContact {
    /// Full name of a parent, adoptive parent or custodian.
    pub full_name: String,
    pub email: String,
}

pub struct CleanContact {
    pub full_name: String,
    pub email: String,
    pub bidx: Vec<u8>,
}

/// Field names differ between registration (`minor.guardian.*`) and the cabinet.
pub fn clean_contact(
    crypto: &Crypto,
    c: &GuardianContact,
    own_email: &str,
    v: &mut Validator,
    fields: (&'static str, &'static str),
) -> CleanContact {
    let full_name = c.full_name.split_whitespace().collect::<Vec<_>>().join(" ");
    let email = normalize_email(&c.email);
    let chars = full_name.chars().count();
    v.check(
        (2..=200).contains(&chars),
        fields.0,
        "Укажите ФИО законного представителя",
    )
    .check(valid_email(&email), fields.1, "Некорректный email")
    .check(
        email != normalize_email(own_email),
        fields.1,
        "Нужна почта взрослого, а не ваша",
    );
    CleanContact {
        bidx: crypto.blind_index(&email),
        full_name,
        email,
    }
}

pub fn state(
    sent_at: Option<DateTime<Utc>>,
    link_expires_at: Option<DateTime<Utc>>,
    decision: Option<GuardianDecision>,
    revoked_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> GuardianState {
    match decision {
        Some(GuardianDecision::Granted) if revoked_at.is_some() => GuardianState::Revoked,
        Some(GuardianDecision::Granted) => GuardianState::Granted,
        Some(GuardianDecision::Declined) => GuardianState::Declined,
        None if sent_at.is_none() => GuardianState::NotSent,
        None if link_expires_at.is_none_or(|e| e <= now) => GuardianState::Expired,
        None => GuardianState::Pending,
    }
}

pub fn request_state(r: &db::guardians::GuardianRequest, now: DateTime<Utc>) -> GuardianState {
    state(
        r.sent_at,
        r.confirm_expires_at,
        r.decision,
        r.revoked_at,
        now,
    )
}

/// Enough to recognize one's own child without handing the address to a stranger who got the
/// letter by mistake: `an***@mail.ru`.
pub fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return "***".into();
    };
    let keep = if local.chars().count() > 3 { 2 } else { 1 };
    let head: String = local.chars().take(keep).collect();
    format!("{head}***@{domain}")
}

pub fn letter(
    public_url: &str,
    guardian_name: &str,
    participant: &str,
    confirm: &str,
    revoke: &str,
) -> String {
    format!(
        "Здравствуйте, {guardian_name}!\n\n\
         На «ФСП Таланты» создан аккаунт {participant}, и его владелец указал вас как законного представителя. \
         «ФСП Таланты» — платформа подбора ИТ-специалистов Федерации спортивного программирования России: \
         участник проходит тест по своему направлению, а компании сами предлагают работу или стажировку и сразу называют условия.\n\n\
         Участнику от 14 до 18 лет, поэтому компании увидят его анонимный профиль только с вашего согласия. \
         Пройти тест он может и без него.\n\n\
         Прочитать, на что вы соглашаетесь, и ответить:\n{public_url}/guardian/confirm?token={confirm}\n\
         Ссылка действует {CONFIRM_TTL_DAYS} дней и срабатывает один раз.\n\n\
         Если позже передумаете, отозвать согласие можно по этой ссылке — профиль сразу скроется из поиска:\n\
         {public_url}/guardian/confirm?token={revoke}\n\n\
         Если вы не знаете, о ком речь, ничего не делайте: без вашего согласия профиль не покажут компаниям.\n\n\
         — ФСП Таланты"
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Three letters an hour per minor, five a day per recipient: the platform cannot be used to flood
/// someone's inbox from a batch of fake accounts.
pub async fn letter_quota(state: &AppState, candidate: Uuid, email_bidx: &[u8]) -> ApiResult<()> {
    let per_candidate = state
        .kv
        .hit(
            &format!("rl:guardian:{candidate}"),
            Duration::from_secs(3600),
        )
        .await?;
    let per_recipient = state
        .kv
        .hit(
            &format!("rl:guardian-to:{}", hex(email_bidx)),
            Duration::from_secs(86_400),
        )
        .await?;
    let wait = [(per_candidate, 3), (per_recipient, 5)]
        .into_iter()
        .filter(|(hit, limit)| hit.over(*limit))
        .map(|(hit, _)| hit.retry_after)
        .max();
    match wait {
        Some(retry_after) => Err(ApiError::rate_limited("guardian_letters", retry_after)),
        None => Ok(()),
    }
}

/// Issues a new link pair for the request (earlier letters stop working) and sends the letter.
pub async fn send_letter(
    state: &AppState,
    candidate: Uuid,
    request: &db::guardians::GuardianRequest,
) -> ApiResult<()> {
    let (Some(name_enc), Some(email_enc)) = (&request.full_name_enc, &request.email_enc) else {
        return Err(ApiError::conflict(
            "guardian_not_needed",
            "Согласие представителя больше не нужно",
        ));
    };
    let account = db::users::user_by_id(&state.pool, candidate)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    let (confirm, confirm_hash) = new_token();
    let (revoke, revoke_hash) = new_token();
    let expires = Utc::now() + chrono::Duration::days(CONFIRM_TTL_DAYS);
    if db::guardians::set_letter(
        &state.pool,
        request.id,
        &confirm_hash,
        &revoke_hash,
        expires,
    )
    .await?
        == 0
    {
        return Err(ApiError::conflict(
            "guardian_already_answered",
            "Представитель уже ответил на это письмо",
        ));
    }
    let body = letter(
        &state.config.public_url,
        &state.crypto.decrypt_str(name_enc)?,
        &mask_email(&state.crypto.decrypt_str(&account.email_enc)?),
        &confirm,
        &revoke,
    );
    let to = state.crypto.decrypt_str(email_enc)?;
    state.mailer.send(&to, SUBJECT, &body).await?;
    Ok(())
}

/// Called once the minor confirms their own email: an account nobody controls cannot make the
/// platform write to a third party.
pub async fn after_email_verified(state: &AppState, user_id: Uuid) {
    if let Err(e) = first_letter(state, user_id).await {
        tracing::error!(error = %e, user = %user_id, "guardian letter not sent");
    }
}

async fn first_letter(state: &AppState, user_id: Uuid) -> ApiResult<()> {
    let Some(age) = db::guardians::candidate_age(&state.pool, user_id).await? else {
        return Ok(());
    };
    if !age.minor {
        return Ok(());
    }
    let Some(request) = db::guardians::latest_request(&state.pool, user_id).await? else {
        return Ok(());
    };
    if request.sent_at.is_some() || request.decision.is_some() {
        return Ok(());
    }
    letter_quota(state, user_id, request.email_bidx.as_deref().unwrap_or(&[])).await?;
    send_letter(state, user_id, &request).await
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SweepReport {
    pub requests: u64,
    pub grants: u64,
    pub birth_dates: u64,
}

/// Retention: 30 days after the 18th birthday the representative's name, email, links, IP and
/// browser are erased, then the birth date. The grant row keeps the document version and dates.
pub async fn sweep(pool: &PgPool) -> Result<SweepReport, sqlx::Error> {
    let days = GUARDIAN_RETENTION_DAYS as i32;
    let mut tx = pool.begin().await?;
    let requests = db::guardians::erase_expired_requests(&mut *tx, days).await?;
    let grants = db::guardians::erase_expired_grant_trails(&mut *tx, days).await?;
    let birth_dates = db::guardians::clear_expired_birth_dates(&mut *tx, days).await?;
    tx.commit().await?;
    Ok(SweepReport {
        requests,
        grants,
        birth_dates,
    })
}

/// At startup and then every 6 hours: a day's delay is fine for a 30-day rule.
pub fn spawn_sweeper(pool: PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(SWEEP_EVERY);
        loop {
            tick.tick().await;
            match sweep(&pool).await {
                Ok(r) if r.requests > 0 || r.grants > 0 || r.birth_dates > 0 => {
                    tracing::info!(
                        requests = r.requests,
                        grants = r.grants,
                        birth_dates = r.birth_dates,
                        "guardian data retention sweep"
                    );
                }
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "guardian data retention sweep failed"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_follows_the_answer() {
        let now = Utc::now();
        let later = now + chrono::Duration::days(1);
        let earlier = now - chrono::Duration::days(1);
        assert_eq!(state(None, None, None, None, now), GuardianState::NotSent);
        assert_eq!(
            state(Some(earlier), Some(later), None, None, now),
            GuardianState::Pending
        );
        assert_eq!(
            state(Some(earlier), Some(earlier), None, None, now),
            GuardianState::Expired
        );
        assert_eq!(
            state(
                Some(earlier),
                Some(earlier),
                Some(GuardianDecision::Granted),
                None,
                now
            ),
            GuardianState::Granted,
            "an answered link no longer expires"
        );
        assert_eq!(
            state(
                Some(earlier),
                Some(later),
                Some(GuardianDecision::Granted),
                Some(now),
                now
            ),
            GuardianState::Revoked
        );
        assert_eq!(
            state(
                Some(earlier),
                Some(later),
                Some(GuardianDecision::Declined),
                None,
                now
            ),
            GuardianState::Declined
        );
    }

    #[test]
    fn masked_email_keeps_only_a_hint() {
        assert_eq!(mask_email("anna.petrova@mail.ru"), "an***@mail.ru");
        assert_eq!(mask_email("ivo@ya.ru"), "i***@ya.ru");
        assert_eq!(mask_email("broken"), "***");
    }

    #[test]
    fn letter_carries_both_links_of_the_family() {
        let text = letter(
            "https://talents.example",
            "Мария Петровна",
            "an***@mail.ru",
            "CONFIRM",
            "REVOKE",
        );
        assert!(text.contains("https://talents.example/guardian/confirm?token=CONFIRM"));
        assert!(text.contains("https://talents.example/guardian/confirm?token=REVOKE"));
        assert!(text.contains("an***@mail.ru") && text.contains("7 дней"));
    }

    #[test]
    fn contact_must_be_someone_else() {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let mut v = Validator::default();
        let c = GuardianContact {
            full_name: "  Петрова   Мария  Ивановна ".into(),
            email: " Mama@Example.RU ".into(),
        };
        let clean = clean_contact(&crypto, &c, "kid@example.ru", &mut v, ("n", "e"));
        assert!(v.finish().is_ok());
        assert_eq!(clean.full_name, "Петрова Мария Ивановна");
        assert_eq!(clean.email, "mama@example.ru");
        assert_eq!(clean.bidx, crypto.blind_index("mama@example.ru"));

        let mut v = Validator::default();
        let own = GuardianContact {
            full_name: "Я".into(),
            email: "KID@example.ru".into(),
        };
        clean_contact(&crypto, &own, "kid@example.ru", &mut v, ("n", "e"));
        let Err(ApiError::Validation(errors)) = v.finish() else {
            panic!("own address and a one-letter name must fail")
        };
        let json = serde_json::to_value(&errors).unwrap();
        assert!(json["n"].is_array() && json["e"].is_array());
    }
}
