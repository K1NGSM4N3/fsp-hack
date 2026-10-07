//! Consent of a minor candidate's legal representative: the public page behind the letter's two
//! links, and the minor's own view with «Отправить письмо ещё раз» and «Изменить адрес».

use axum::{extract::State, http::HeaderMap};
use chrono::{DateTime, NaiveDate, Utc};
use db::types::GuardianDecision;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::catalog::ConsentDocument;
use crate::{
    age,
    auth::{CandidateUser, client_ip, user_agent},
    crypto::{Crypto, sha256},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Json, Query},
    guardian::{self, GuardianContact, GuardianState, clean_contact},
    notify::{GuardianOutcome, Notifier},
    state::AppState,
};

pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(view, answer))
}

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(status, change))
        .routes(routes!(resend))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GuardianLink {
    /// One-time answer link, valid for 7 days.
    Confirm,
    /// Revocation link of the same letter.
    Revoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GuardianAction {
    Grant,
    Decline,
    Revoke,
}

#[derive(Serialize, ToSchema)]
pub struct GuardianConsentView {
    pub link: GuardianLink,
    pub state: GuardianState,
    /// The participant has turned 18: the consent is no longer needed and nothing can be done.
    pub participant_adult: bool,
    /// What this link can do right now.
    pub actions: Vec<GuardianAction>,
    /// Masked email of the participant's account, e.g. `an***@mail.ru`.
    pub participant: String,
    /// As the participant entered it.
    pub guardian_name: Option<String>,
    pub link_expires_at: Option<DateTime<Utc>>,
    pub answered_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// Current text of `guardian_consent`.
    pub document: ConsentDocument,
}

#[derive(Deserialize, IntoParams)]
pub struct TokenQuery {
    /// Token from either link of the letter.
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GuardianAnswer {
    pub token: String,
    pub action: GuardianAction,
}

pub fn actions(link: GuardianLink, state: GuardianState, minor: bool) -> Vec<GuardianAction> {
    use GuardianAction::{Decline, Grant, Revoke};
    if !minor {
        return Vec::new();
    }
    match (link, state) {
        (GuardianLink::Confirm, GuardianState::Pending) => vec![Grant, Decline],
        (GuardianLink::Revoke, GuardianState::Pending | GuardianState::Expired) => vec![Decline],
        (GuardianLink::Revoke, GuardianState::Granted) => vec![Revoke],
        _ => Vec::new(),
    }
}

fn used() -> ApiError {
    ApiError::conflict("token_used", "По этой ссылке уже ответили")
        .with_detail("Чтобы отозвать согласие, откройте вторую ссылку из письма")
}

fn refusal(link: GuardianLink, state: GuardianState, minor: bool) -> ApiError {
    if !minor {
        return ApiError::conflict(
            "guardian_not_needed",
            "Участнику уже исполнилось 18 — согласие представителя больше не нужно",
        );
    }
    match (link, state) {
        (GuardianLink::Confirm, GuardianState::Expired) => {
            ApiError::bad_request("token_expired", "Ссылка устарела")
                .with_detail("Попросите участника отправить письмо ещё раз из личного кабинета")
        }
        (GuardianLink::Confirm, _) => used(),
        (GuardianLink::Revoke, GuardianState::Declined | GuardianState::Revoked) => {
            ApiError::conflict(
                "token_used",
                "Согласия уже нет: вы отказались или отозвали его",
            )
        }
        (GuardianLink::Revoke, _) => ApiError::conflict(
            "action_unavailable",
            "Подтвердить согласие можно по первой ссылке из письма",
        ),
    }
}

fn link_of(t: &db::guardians::LinkTarget) -> GuardianLink {
    if t.confirm_link {
        GuardianLink::Confirm
    } else {
        GuardianLink::Revoke
    }
}

fn target_state(t: &db::guardians::LinkTarget, now: DateTime<Utc>) -> GuardianState {
    guardian::state(
        t.sent_at,
        t.confirm_expires_at,
        t.decision,
        t.revoked_at,
        now,
    )
}

async fn target(state: &AppState, token: &str) -> ApiResult<db::guardians::LinkTarget> {
    db::guardians::link_target(&state.pool, &sha256(token.trim().as_bytes()))
        .await?
        .ok_or_else(|| {
            ApiError::bad_request("invalid_token", "Ссылка недействительна")
                .with_detail("Откройте ссылку из последнего письма целиком")
        })
}

async fn build_view(
    state: &AppState,
    t: db::guardians::LinkTarget,
    now: DateTime<Utc>,
) -> ApiResult<GuardianConsentView> {
    let document = db::consents::current_document(&state.pool, db::guardians::KIND)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let link = link_of(&t);
    let current = target_state(&t, now);
    Ok(GuardianConsentView {
        link,
        state: current,
        participant_adult: !t.minor,
        actions: actions(link, current, t.minor),
        participant: guardian::mask_email(&state.crypto.decrypt_str(&t.account_email_enc)?),
        guardian_name: t
            .full_name_enc
            .as_deref()
            .map(|b| state.crypto.decrypt_str(b))
            .transpose()?,
        link_expires_at: t.confirm_expires_at.filter(|_| t.decision.is_none()),
        answered_at: t.answered_at,
        revoked_at: t.revoked_at,
        document: document.into(),
    })
}

/// What a link from the representative's letter is about: the consent text, who asked, the
/// current answer and what this link can do now. Anonymous: the token is the credential
#[utoipa::path(get, path = "/guardian/consent", tag = "guardian",
    operation_id = "guardian_consent",
    params(TokenQuery),
    responses(
        (status = 200, body = GuardianConsentView),
        (status = 400, description = "`invalid_token`: unknown token or a link of a superseded letter", body = Problem),
    ))]
async fn view(
    State(state): State<AppState>,
    Query(q): Query<TokenQuery>,
) -> ApiResult<Json<GuardianConsentView>> {
    let t = target(&state, &q.token).await?;
    Ok(Json(build_view(&state, t, Utc::now()).await?))
}

/// The representative's answer. The answer link grants or declines once while it is valid; the
/// revocation link withdraws the consent (or declines before any answer). Granting writes a
/// `guardian_consent` grant on the minor's behalf with the representative's IP, User-Agent and
/// time; declining or revoking hides the profile
#[utoipa::path(post, path = "/guardian/consent", tag = "guardian",
    operation_id = "answer_guardian_consent",
    request_body = GuardianAnswer,
    responses(
        (status = 200, body = GuardianConsentView),
        (status = 400, description = "`invalid_token`; `token_expired`: the answer link is older than 7 days", body = Problem),
        (status = 409, description = "`token_used`: already answered or revoked; `action_unavailable`: granting needs the answer link; `guardian_not_needed`: the participant is 18", body = Problem),
    ))]
async fn answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<GuardianAnswer>,
) -> ApiResult<Json<GuardianConsentView>> {
    let t = target(&state, &req.token).await?;
    let link = link_of(&t);
    let current = target_state(&t, Utc::now());
    if !actions(link, current, t.minor).contains(&req.action) {
        return Err(refusal(link, current, t.minor));
    }
    let mut tx = state.pool.begin().await?;
    match req.action {
        GuardianAction::Grant => {
            let doc = db::consents::current_document(&mut *tx, db::guardians::KIND)
                .await?
                .ok_or(sqlx::Error::RowNotFound)?;
            let grant = db::consents::grant(
                &mut *tx,
                Uuid::now_v7(),
                t.candidate_id,
                doc.id,
                client_ip(&headers).as_deref(),
                user_agent(&headers).as_deref(),
            )
            .await?;
            if db::guardians::answer(
                &mut *tx,
                t.request_id,
                GuardianDecision::Granted,
                Some(grant.id),
            )
            .await?
                == 0
            {
                return Err(used());
            }
        }
        GuardianAction::Decline => {
            if db::guardians::answer(&mut *tx, t.request_id, GuardianDecision::Declined, None)
                .await?
                == 0
            {
                return Err(used());
            }
            db::me::hide_candidate(&mut *tx, t.candidate_id).await?;
        }
        GuardianAction::Revoke => {
            let Some(consent) = t.consent_id else {
                return Err(used());
            };
            if db::guardians::revoke_grant(&mut *tx, consent).await? == 0 {
                return Err(used());
            }
            db::me::hide_candidate(&mut *tx, t.candidate_id).await?;
        }
    }
    let outcome = match req.action {
        GuardianAction::Grant => GuardianOutcome::Granted,
        GuardianAction::Decline => GuardianOutcome::Declined,
        GuardianAction::Revoke => GuardianOutcome::Revoked,
    };
    Notifier::new(&state)
        .guardian(&mut *tx, t.candidate_id, outcome)
        .await?;
    tx.commit().await?;
    state.candidates.invalidate().await;
    let t = target(&state, &req.token).await?;
    Ok(Json(build_view(&state, t, Utc::now()).await?))
}

#[derive(Serialize, ToSchema)]
pub struct GuardianStatus {
    /// Under 18 today: employers see the profile only with the representative's consent.
    pub required: bool,
    /// The 18th birthday, when every restriction lifts by itself.
    pub adult_from: Option<NaiveDate>,
    /// `None` when not required.
    pub state: Option<GuardianState>,
    pub guardian_name: Option<String>,
    pub guardian_email: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub link_expires_at: Option<DateTime<Utc>>,
    pub answered_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub(crate) async fn load_status(state: &AppState, candidate: Uuid) -> ApiResult<GuardianStatus> {
    let age = db::guardians::candidate_age(&state.pool, candidate)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let request = if age.minor {
        db::guardians::latest_request(&state.pool, candidate).await?
    } else {
        None
    };
    status_view(age, request.as_ref(), &state.crypto, Utc::now())
}

fn status_view(
    age: db::guardians::CandidateAge,
    request: Option<&db::guardians::GuardianRequest>,
    crypto: &Crypto,
    now: DateTime<Utc>,
) -> ApiResult<GuardianStatus> {
    let mut out = GuardianStatus {
        required: age.minor,
        adult_from: age.birth_date.filter(|_| age.minor).map(age::adult_from),
        state: None,
        guardian_name: None,
        guardian_email: None,
        sent_at: None,
        link_expires_at: None,
        answered_at: None,
        revoked_at: None,
    };
    if !age.minor {
        return Ok(out);
    }
    if let Some(r) = request {
        let decrypt = |b: &Option<Vec<u8>>| b.as_deref().map(|b| crypto.decrypt_str(b)).transpose();
        out.state = Some(guardian::request_state(r, now));
        out.guardian_name = decrypt(&r.full_name_enc)?;
        out.guardian_email = decrypt(&r.email_enc)?;
        out.sent_at = r.sent_at;
        out.link_expires_at = r.confirm_expires_at.filter(|_| r.decision.is_none());
        out.answered_at = r.answered_at;
        out.revoked_at = r.revoked_at;
    }
    Ok(out)
}

fn not_needed() -> ApiError {
    ApiError::conflict(
        "guardian_not_needed",
        "Вам уже есть 18 — согласие представителя не нужно",
    )
}

fn already_confirmed() -> ApiError {
    ApiError::conflict(
        "guardian_already_confirmed",
        "Законный представитель уже подтвердил согласие",
    )
}

/// The current request of a minor; `None` for adults.
async fn minor_request(
    state: &AppState,
    candidate: Uuid,
) -> ApiResult<Option<db::guardians::GuardianRequest>> {
    let age = db::guardians::candidate_age(&state.pool, candidate)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !age.minor {
        return Err(not_needed());
    }
    let request = db::guardians::latest_request(&state.pool, candidate).await?;
    if request
        .as_ref()
        .is_some_and(|r| guardian::request_state(r, Utc::now()) == GuardianState::Granted)
    {
        return Err(already_confirmed());
    }
    Ok(request)
}

/// Whether your legal representative has confirmed the consent: while you are under 18, employers
/// see the profile only after that. `required: false` for adults
#[utoipa::path(get, path = "/guardian", tag = "candidate",
    responses((status = 200, body = GuardianStatus), (status = 401, body = Problem), (status = 403, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn status(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<GuardianStatus>> {
    Ok(Json(load_status(&state, user.id).await?))
}

/// Send the letter to the representative again with fresh links (the old ones stop working).
/// After a refusal or revocation it asks the same person again
#[utoipa::path(post, path = "/guardian/resend", tag = "candidate",
    responses(
        (status = 200, body = GuardianStatus),
        (status = 409, description = "`guardian_not_needed`: you are 18 or older; `guardian_already_confirmed`; `guardian_missing`, `guardian_already_answered`", body = Problem),
        (status = 429, description = "Three letters an hour, five a day to one address", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn resend(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<GuardianStatus>> {
    let request = minor_request(&state, user.id)
        .await?
        .ok_or_else(|| ApiError::conflict("guardian_missing", "Укажите законного представителя"))?;
    let (Some(name), Some(email), Some(bidx)) = (
        &request.full_name_enc,
        &request.email_enc,
        &request.email_bidx,
    ) else {
        return Err(not_needed());
    };
    guardian::letter_quota(&state, user.id, bidx).await?;
    let request = if request.decision.is_some() {
        db::guardians::insert_request(&state.pool, Uuid::now_v7(), user.id, name, email, bidx)
            .await?;
        db::guardians::latest_request(&state.pool, user.id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?
    } else {
        request
    };
    guardian::send_letter(&state, user.id, &request).await?;
    Ok(Json(load_status(&state, user.id).await?))
}

/// Change the representative's name or email and send them the letter. Links sent to the
/// previous address stop working at once
#[utoipa::path(put, path = "/guardian", tag = "candidate",
    request_body = GuardianContact,
    responses(
        (status = 200, body = GuardianStatus),
        (status = 409, description = "`guardian_not_needed`, `guardian_already_confirmed`; `guardian_already_answered`", body = Problem),
        (status = 422, description = "`validation_failed`: `full_name`, `email` (also when it is your own address)", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn change(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(c): Json<GuardianContact>,
) -> ApiResult<Json<GuardianStatus>> {
    let request = minor_request(&state, user.id).await?;
    let account = db::users::user_by_id(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    let own = state.crypto.decrypt_str(&account.email_enc)?;
    let mut v = Validator::default();
    let clean = clean_contact(&state.crypto, &c, &own, &mut v, ("full_name", "email"));
    v.finish()?;
    guardian::letter_quota(&state, user.id, &clean.bidx).await?;
    let name_enc = state.crypto.encrypt_str(&clean.full_name)?;
    let email_enc = state.crypto.encrypt_str(&clean.email)?;
    let updated = match &request {
        Some(r) if r.decision.is_none() => {
            db::guardians::update_contact(&state.pool, r.id, &name_enc, &email_enc, &clean.bidx)
                .await?
        }
        _ => 0,
    };
    if updated == 0 {
        db::guardians::insert_request(
            &state.pool,
            Uuid::now_v7(),
            user.id,
            &name_enc,
            &email_enc,
            &clean.bidx,
        )
        .await?;
    }
    let request = db::guardians::latest_request(&state.pool, user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    guardian::send_letter(&state, user.id, &request).await?;
    Ok(Json(load_status(&state, user.id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_link_does_its_own_job() {
        use GuardianAction::{Decline, Grant, Revoke};
        use GuardianLink::{Confirm, Revoke as RevokeLink};
        assert_eq!(
            actions(Confirm, GuardianState::Pending, true),
            [Grant, Decline]
        );
        assert!(actions(Confirm, GuardianState::Expired, true).is_empty());
        assert!(
            actions(Confirm, GuardianState::Granted, true).is_empty(),
            "one-time"
        );
        assert_eq!(actions(RevokeLink, GuardianState::Granted, true), [Revoke]);
        assert_eq!(actions(RevokeLink, GuardianState::Expired, true), [Decline]);
        assert!(actions(RevokeLink, GuardianState::Revoked, true).is_empty());
        assert!(
            actions(Confirm, GuardianState::Pending, false).is_empty(),
            "nothing to do once the participant is 18"
        );
    }

    #[test]
    fn refusal_names_why_a_link_cannot_do_it() {
        use GuardianLink::{Confirm, Revoke};
        use GuardianState::{Declined, Expired, Granted, NotSent, Pending, Revoked};
        let code = |link, state, minor| refusal(link, state, minor).summary();
        assert_eq!(code(Confirm, Expired, true), (400, "token_expired", vec![]));
        for state in [Pending, Granted, Declined, Revoked, NotSent] {
            assert_eq!(
                code(Confirm, state, true),
                (409, "token_used", vec![]),
                "{state:?}"
            );
        }
        for state in [Declined, Revoked] {
            assert_eq!(
                code(Revoke, state, true),
                (409, "token_used", vec![]),
                "{state:?}"
            );
        }
        for state in [Pending, Expired, Granted, NotSent] {
            assert_eq!(
                code(Revoke, state, true),
                (409, "action_unavailable", vec![]),
                "{state:?}"
            );
        }
        for link in [Confirm, Revoke] {
            for state in [NotSent, Pending, Expired, Granted, Declined, Revoked] {
                assert_eq!(
                    code(link, state, false),
                    (409, "guardian_not_needed", vec![]),
                    "{link:?} {state:?}"
                );
            }
        }
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn noon() -> DateTime<Utc> {
        "2026-10-07T12:00:00Z".parse().unwrap()
    }

    fn request(crypto: &Crypto, now: DateTime<Utc>) -> db::guardians::GuardianRequest {
        db::guardians::GuardianRequest {
            id: Uuid::now_v7(),
            full_name_enc: Some(crypto.encrypt_str("Петрова Ольга Викторовна").unwrap()),
            email_enc: Some(crypto.encrypt_str("o.petrova@example.ru").unwrap()),
            email_bidx: Some(crypto.blind_index("o.petrova@example.ru")),
            sent_at: Some(now - chrono::Duration::days(1)),
            confirm_expires_at: Some(now + chrono::Duration::days(6)),
            decision: None,
            answered_at: None,
            consent_id: None,
            granted_at: None,
            revoked_at: None,
            erased_at: None,
            created_at: now - chrono::Duration::days(1),
        }
    }

    #[test]
    fn status_shows_the_link_expiry_only_until_answered() {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let now = noon();
        let birth = day(2010, 3, 14);
        let minor = db::guardians::CandidateAge {
            birth_date: Some(birth),
            minor: true,
        };
        let pending = request(&crypto, now);
        let s = status_view(minor, Some(&pending), &crypto, now).unwrap();
        assert!(s.required);
        assert_eq!(s.adult_from, Some(age::adult_from(birth)));
        assert_eq!(s.state, Some(GuardianState::Pending));
        assert_eq!(s.guardian_name.as_deref(), Some("Петрова Ольга Викторовна"));
        assert_eq!(s.guardian_email.as_deref(), Some("o.petrova@example.ru"));
        assert_eq!(s.link_expires_at, pending.confirm_expires_at);

        let answered = db::guardians::GuardianRequest {
            decision: Some(GuardianDecision::Granted),
            answered_at: Some(now),
            ..request(&crypto, now)
        };
        let s = status_view(minor, Some(&answered), &crypto, now).unwrap();
        assert_eq!(s.state, Some(GuardianState::Granted));
        assert_eq!(s.link_expires_at, None);
        assert_eq!(s.answered_at, Some(now));

        let s = status_view(minor, None, &crypto, now).unwrap();
        assert!(s.required && s.state.is_none() && s.guardian_email.is_none());
    }

    #[test]
    fn status_of_an_adult_requires_nothing() {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let now = noon();
        let adult = db::guardians::CandidateAge {
            birth_date: Some(day(2008, 1, 1)),
            minor: false,
        };
        let s = status_view(adult, Some(&request(&crypto, now)), &crypto, now).unwrap();
        assert!(!s.required);
        assert_eq!(s.adult_from, None, "adult_from is shown only to minors");
        assert_eq!(s.state, None);
        assert_eq!(s.guardian_name, None);
        assert_eq!(s.link_expires_at, None);
    }
}
