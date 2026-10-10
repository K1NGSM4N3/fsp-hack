use std::time::Duration;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    AuthUser, REFRESH_COOKIE, REFRESH_TTL_DAYS, Role, clear_session_cookies, client_ip,
    session_cookies, user_agent,
};
use crate::{
    age::{self, AgeBracket},
    crypto::{Crypto, new_token, normalize_email, sha256},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::Json,
    guardian::{self, CleanContact, GuardianContact, clean_contact},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register))
        .routes(routes!(verify_email))
        .routes(routes!(resend_verification))
        .routes(routes!(login))
        .routes(routes!(refresh))
        .routes(routes!(logout))
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: Role,
    /// Required for employers.
    pub company_name: Option<String>,
    /// Accepted documents from `GET /api/v1/consents/current`.
    pub consents: Vec<Uuid>,
    /// Candidates declare their age: `adult` needs nothing else, `minor` (14–17) needs `minor`.
    /// Under 14 the form does not submit. Employers are adults by the declaration in the form's
    /// consent block («Я представляю компанию и мне есть 18 лет»); the field is ignored for them.
    pub age: Option<AgeBracket>,
    /// Required with `age: minor`.
    pub minor: Option<MinorRegistration>,
}

#[derive(Deserialize, ToSchema)]
pub struct MinorRegistration {
    /// Must be 14–17 years ago: the restriction lifts on the 18th birthday.
    pub birth_date: NaiveDate,
    /// Gets a letter once you confirm your own email.
    pub guardian: GuardianContact,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub verification_required: bool,
}

pub(crate) const MIN_PASSWORD_CHARS: usize = 8;
pub(crate) const MAX_PASSWORD_BYTES: usize = 128;

pub(crate) fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && email.len() <= 254
        && !email.chars().any(char::is_whitespace)
}

async fn rate_limit(
    state: &AppState,
    scope: &'static str,
    key: String,
    limit: u64,
    window: Duration,
) -> ApiResult<()> {
    let hit = state.kv.hit(&key, window).await?;
    if hit.over(limit) {
        return Err(ApiError::rate_limited(scope, hit.retry_after));
    }
    Ok(())
}

/// Register a candidate or employer account
#[utoipa::path(post, path = "/register", tag = "auth",
    request_body(content((RegisterRequest = "application/json", examples(
        ("candidate" = (summary = "Соискатель 18+ (id согласий — из GET /consents/current)", value = json!({
            "email": "anna.smirnova@example.ru",
            "password": "Sever-2026!",
            "role": "candidate",
            "age": "adult",
            "consents": ["0191f000-0000-7000-8000-000000000a02", "0191f000-0000-7000-8000-000000000002", "0191f000-0000-7000-8000-000000000003"]
        }))),
        ("minor" = (summary = "Соискатель 14–17 лет: письмо уйдёт законному представителю", value = json!({
            "email": "misha.petrov@example.ru",
            "password": "Shkola-57-2026",
            "role": "candidate",
            "age": "minor",
            "minor": {"birth_date": "2010-03-14", "guardian": {"full_name": "Петрова Ольга Викторовна", "email": "o.petrova@example.ru"}},
            "consents": ["0191f000-0000-7000-8000-000000000a02", "0191f000-0000-7000-8000-000000000002", "0191f000-0000-7000-8000-000000000003"]
        }))),
        ("employer" = (summary = "Компания", value = json!({
            "email": "hr@severny-kod.example",
            "password": "Sever-Kod-2026",
            "role": "employer",
            "company_name": "ООО «Северный код»",
            "consents": ["0191f000-0000-7000-8000-000000000a02", "0191f000-0000-7000-8000-000000000003"]
        }))),
    )))),
    responses(
        (status = 201, body = RegisterResponse),
        (status = 409, description = "`email_taken`", body = Problem),
        (status = 422, description = "`validation_failed`: also `age` for a candidate without it, `minor.birth_date` outside 14–17, `minor.guardian.*`", body = Problem),
        (status = 429, body = Problem),
    ))]
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<RegisterResponse>)> {
    let ip = client_ip(&headers);
    rate_limit(
        &state,
        "register",
        format!("rl:register:{}", ip.as_deref().unwrap_or("-")),
        20,
        Duration::from_secs(3600),
    )
    .await?;

    let email = normalize_email(&req.email);
    let company_name = req.company_name.as_deref().map(str::trim).unwrap_or("");
    let mut v = Validator::default();
    v.check(valid_email(&email), "email", "Некорректный email")
        .check(
            req.password.chars().count() >= MIN_PASSWORD_CHARS,
            "password",
            "Минимум 8 символов",
        )
        .check(
            req.password.len() <= MAX_PASSWORD_BYTES,
            "password",
            "Слишком длинный пароль",
        )
        .check(req.role != Role::Admin, "role", "Недопустимая роль")
        .check(
            req.role != Role::Employer || !company_name.is_empty(),
            "company_name",
            "Укажите название компании",
        );
    let (age_bracket, minor) =
        check_age(&state.crypto, &req, &email, Utc::now().date_naive(), &mut v);
    v.finish()?;

    let docs = db::consents::current_consent_documents(&state.pool).await?;
    let role_db: db::types::UserRole = req.role.into();
    require_consents(&docs, role_db, &req.consents)?;

    let bidx = state.crypto.blind_index(&email);
    if db::users::user_by_email_bidx(&state.pool, &bidx)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict(
            "email_taken",
            "Этот email уже зарегистрирован",
        ));
    }

    let password = req.password.clone();
    let crypto = state.crypto.clone();
    let password_hash =
        tokio::task::spawn_blocking(move || crypto.passwords.hash(&password)).await??;

    let user_id = Uuid::now_v7();
    let (token, token_hash) = new_token();
    let ua = user_agent(&headers);
    let mut tx = state.pool.begin().await?;
    // The pre-check above races with a concurrent sign-up (double submit); the unique index is the real guard.
    db::users::insert_user(
        &mut *tx,
        user_id,
        role_db,
        &state.crypto.encrypt_str(&email)?,
        &bidx,
        &password_hash,
    )
    .await
    .map_err(|e| {
        if db::is_unique_violation(&e) {
            ApiError::conflict("email_taken", "Этот email уже зарегистрирован")
        } else {
            e.into()
        }
    })?;
    for doc in docs.iter().filter(|d| req.consents.contains(&d.id)) {
        db::consents::insert_registration_consent(
            &mut *tx,
            Uuid::now_v7(),
            user_id,
            doc.id,
            ip.as_deref(),
            ua.as_deref(),
            age_bracket.into(),
        )
        .await?;
    }
    match req.role {
        Role::Candidate => {
            crate::public_code::insert_candidate(&mut tx, user_id).await?;
            if let Some((birth_date, contact)) = &minor {
                db::guardians::set_birth_date(&mut *tx, user_id, *birth_date).await?;
                db::guardians::insert_request(
                    &mut *tx,
                    Uuid::now_v7(),
                    user_id,
                    &state.crypto.encrypt_str(&contact.full_name)?,
                    &state.crypto.encrypt_str(&contact.email)?,
                    &contact.bidx,
                )
                .await?;
            }
        }
        Role::Employer => {
            db::companies::insert_company(&mut *tx, Uuid::now_v7(), user_id, company_name).await?;
        }
        Role::Admin => return Err(ApiError::field("role", "Недопустимая роль")),
    }
    db::users::insert_email_verification(
        &mut *tx,
        &token_hash,
        user_id,
        Utc::now() + chrono::Duration::hours(24),
    )
    .await?;
    tx.commit().await?;

    send_verification(&state, &email, &token).await;
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id,
            verification_required: true,
        }),
    ))
}

/// The declared age: employers are adults by the form's declaration, a candidate says it, and
/// «Мне 14–17 лет» needs a birth date that really is 14–17 `today` and an adult's address that is
/// not the candidate's own. The bracket falls back to adult only when `v` already holds an error.
fn check_age(
    crypto: &Crypto,
    req: &RegisterRequest,
    email: &str,
    today: NaiveDate,
    v: &mut Validator,
) -> (AgeBracket, Option<(NaiveDate, CleanContact)>) {
    let age_bracket = match req.role {
        Role::Candidate => req.age,
        _ => Some(AgeBracket::Adult),
    };
    v.check(age_bracket.is_some(), "age", "Укажите, есть ли вам 18 лет");
    let minor = match (age_bracket, &req.minor) {
        (Some(AgeBracket::Minor), Some(m)) => {
            if let Err(e) = age::check_minor_birth_date(m.birth_date, today) {
                v.check(false, "minor.birth_date", e.message());
            }
            let contact = clean_contact(
                crypto,
                &m.guardian,
                email,
                v,
                ("minor.guardian.full_name", "minor.guardian.email"),
            );
            Some((m.birth_date, contact))
        }
        (Some(AgeBracket::Minor), None) => {
            v.check(
                false,
                "minor",
                "Укажите дату рождения и законного представителя",
            );
            None
        }
        _ => None,
    };
    (age_bracket.unwrap_or(AgeBracket::Adult), minor)
}

/// Every current document required for `role` is among `accepted`; otherwise 422 on `consents`
/// listing the titles of the rest.
fn require_consents(
    docs: &[db::consents::ConsentDocument],
    role: db::types::UserRole,
    accepted: &[Uuid],
) -> ApiResult<()> {
    let missing: Vec<&str> = docs
        .iter()
        .filter(|d| d.required_for.contains(&role) && !accepted.contains(&d.id))
        .map(|d| d.title.as_str())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(ApiError::field(
        "consents",
        format!("Нужно согласие: {}", missing.join(", ")),
    ))
}

async fn send_verification(state: &AppState, email: &str, token: &str) {
    let link = format!("{}/verify-email?token={token}", state.config.public_url);
    let body = format!(
        "Здравствуйте!\n\nПодтвердите email, чтобы завершить регистрацию:\n{link}\n\nСсылка действует 24 часа. \
         Если вы не регистрировались — просто проигнорируйте письмо."
    );
    if let Err(e) = state.mailer.send(email, "Подтверждение email", &body).await {
        tracing::error!(error = ?e, "failed to send verification email");
    }
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// Confirm the email address with the token from the letter. For a candidate who registered as
/// 14–17 this also sends the legal representative their letter
#[utoipa::path(post, path = "/verify-email", tag = "auth",
    request_body = VerifyEmailRequest,
    responses((status = 204), (status = 400, description = "`invalid_token`", body = Problem)))]
async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> ApiResult<StatusCode> {
    let user_id =
        db::users::consume_email_verification(&state.pool, &sha256(req.token.trim().as_bytes()))
            .await?
            .ok_or_else(|| {
                ApiError::bad_request("invalid_token", "Ссылка недействительна или устарела")
            })?;
    if db::users::mark_email_verified(&state.pool, user_id).await? > 0 {
        guardian::after_email_verified(&state, user_id).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct ResendRequest {
    pub email: String,
}

/// Send the confirmation letter again (always 204: does not reveal whether the email exists)
#[utoipa::path(post, path = "/resend-verification", tag = "auth",
    request_body = ResendRequest, responses((status = 204), (status = 429, body = Problem)))]
async fn resend_verification(
    State(state): State<AppState>,
    Json(req): Json<ResendRequest>,
) -> ApiResult<StatusCode> {
    let email = normalize_email(&req.email);
    rate_limit(
        &state,
        "resend_verification",
        format!("rl:resend:{email}"),
        3,
        Duration::from_secs(3600),
    )
    .await?;
    let bidx = state.crypto.blind_index(&email);
    if let Some(user) = db::users::user_by_email_bidx(&state.pool, &bidx).await?
        && user.email_verified_at.is_none()
        && user.deleted_at.is_none()
    {
        let (token, token_hash) = new_token();
        db::users::insert_email_verification(
            &state.pool,
            &token_hash,
            user.id,
            Utc::now() + chrono::Duration::hours(24),
        )
        .await?;
        send_verification(&state, &email, &token).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"email": "candidate@demo.fsp", "password": "demo12345"}))]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct SessionUser {
    pub id: Uuid,
    pub role: Role,
    pub email: String,
    pub email_verified: bool,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    /// Same token as the `fsp_at` cookie, for Bearer clients and Swagger.
    pub access_token: String,
    pub expires_in: i64,
    pub user: SessionUser,
}

/// Log in: sets `fsp_at`, `fsp_rt` and `fsp_session` httpOnly cookies
#[utoipa::path(post, path = "/login", tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, description = "`invalid_credentials`", body = Problem),
        (status = 403, description = "`email_not_verified`", body = Problem),
        (status = 429, body = Problem),
    ))]
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> ApiResult<(CookieJar, Json<LoginResponse>)> {
    let email = normalize_email(&req.email);
    let ip = client_ip(&headers).unwrap_or_default();
    rate_limit(
        &state,
        "login",
        format!("rl:login:{ip}:{email}"),
        10,
        Duration::from_secs(900),
    )
    .await?;

    let user = db::users::user_by_email_bidx(&state.pool, &state.crypto.blind_index(&email))
        .await?
        .filter(|u| u.deleted_at.is_none());

    // Unknown emails still pay for one Argon2 verification: response time must not reveal accounts.
    let phc = match &user {
        Some(u) => u.password_hash.clone(),
        None => crate::crypto::dummy_hash().unwrap_or_default().to_owned(),
    };
    let password = req.password.clone();
    let crypto = state.crypto.clone();
    let ok = tokio::task::spawn_blocking(move || crypto.passwords.verify(&password, &phc)).await?;
    let user = match user {
        Some(u) if ok => u,
        _ => {
            return Err(ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "Неверный email или пароль",
            ));
        }
    };
    if user.email_verified_at.is_none() {
        return Err(ApiError::forbidden(
            "email_not_verified",
            "Подтвердите email — мы отправили письмо со ссылкой",
        ));
    }
    db::users::touch_last_login(&state.pool, user.id).await?;

    let role = Role::from(user.role);
    let (jar, access_token) = start_session(&state, jar, user.id, role, Uuid::now_v7()).await?;
    Ok((
        jar,
        Json(LoginResponse {
            access_token,
            expires_in: super::jwt::ACCESS_TTL_SECS,
            user: SessionUser {
                id: user.id,
                role,
                email,
                email_verified: true,
            },
        }),
    ))
}

async fn start_session(
    state: &AppState,
    jar: CookieJar,
    user_id: Uuid,
    role: Role,
    family_id: Uuid,
) -> ApiResult<(CookieJar, String)> {
    let now = Utc::now();
    let access = state.jwt.issue(user_id, role, now.timestamp())?;
    let (refresh, refresh_hash) = new_token();
    db::users::insert_refresh_token(
        &state.pool,
        Uuid::now_v7(),
        user_id,
        family_id,
        &refresh_hash,
        now + chrono::Duration::days(REFRESH_TTL_DAYS),
    )
    .await?;
    let jar = session_cookies(
        jar,
        access.clone(),
        now.timestamp() + super::jwt::ACCESS_TTL_SECS,
        refresh,
        state.config.cookie_secure,
    );
    Ok((jar, access))
}

#[derive(Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

/// Rotate the refresh token (cookie `fsp_rt`) and issue a new access token
#[utoipa::path(post, path = "/refresh", tag = "auth",
    responses((status = 200, body = RefreshResponse), (status = 401, body = Problem)))]
async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> ApiResult<(CookieJar, Json<RefreshResponse>)> {
    let token = jar
        .get(REFRESH_COOKIE)
        .map(|c| c.value().to_owned())
        .ok_or_else(ApiError::unauthorized)?;
    let row = db::users::refresh_token_by_hash(&state.pool, &sha256(token.as_bytes()))
        .await?
        .ok_or_else(ApiError::unauthorized)?;

    if row.revoked_at.is_some() || row.expires_at < Utc::now() {
        return Err(ApiError::unauthorized());
    }
    // A rotated token presented again means it was copied: revoke the whole family.
    let rotated = db::users::rotate_refresh_token(&state.pool, row.id).await?;
    if row.rotated_at.is_some() || rotated == 0 {
        db::users::revoke_refresh_family(&state.pool, row.family_id).await?;
        tracing::warn!(user_id = %row.user_id, "refresh token reuse detected, family revoked");
        return Err(ApiError::unauthorized());
    }
    let user = db::users::user_by_id(&state.pool, row.user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if user.deleted_at.is_some() {
        return Err(ApiError::unauthorized());
    }
    let (jar, access_token) =
        start_session(&state, jar, user.id, user.role.into(), row.family_id).await?;
    Ok((
        jar,
        Json(RefreshResponse {
            access_token,
            expires_in: super::jwt::ACCESS_TTL_SECS,
        }),
    ))
}

/// Log out: revoke the refresh family, deny-list the access token, clear cookies
#[utoipa::path(post, path = "/logout", tag = "auth", responses((status = 204)))]
async fn logout(
    State(state): State<AppState>,
    user: Option<AuthUser>,
    jar: CookieJar,
) -> ApiResult<(CookieJar, StatusCode)> {
    if let Some(token) = jar.get(REFRESH_COOKIE) {
        if let Some(row) =
            db::users::refresh_token_by_hash(&state.pool, &sha256(token.value().as_bytes())).await?
        {
            db::users::revoke_refresh_family(&state.pool, row.family_id).await?;
        }
    }
    if let Some(user) = user {
        user.revoke(&state).await?;
    }
    Ok((clear_session_cookies(jar), StatusCode::NO_CONTENT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_validation() {
        assert!(valid_email("ivan@example.ru"));
        assert!(!valid_email("ivan@example"));
        assert!(!valid_email("ivan example@x.ru"));
        assert!(!valid_email("@x.ru"));
    }

    const OWN_EMAIL: &str = "misha.petrov@example.ru";

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 7).unwrap()
    }

    fn form(
        role: Role,
        age: Option<AgeBracket>,
        minor: Option<(NaiveDate, &str)>,
    ) -> RegisterRequest {
        RegisterRequest {
            email: OWN_EMAIL.into(),
            password: "Shkola-57-2026".into(),
            role,
            company_name: None,
            consents: Vec::new(),
            age,
            minor: minor.map(|(birth_date, guardian_email)| MinorRegistration {
                birth_date,
                guardian: GuardianContact {
                    full_name: "Петрова Ольга Викторовна".into(),
                    email: guardian_email.into(),
                },
            }),
        }
    }

    /// The registration's age checks: the bracket, the minor's data, and the field errors as JSON.
    fn age_of(
        req: &RegisterRequest,
    ) -> (
        AgeBracket,
        Option<(NaiveDate, CleanContact)>,
        serde_json::Value,
    ) {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let mut v = Validator::default();
        let (bracket, minor) = check_age(&crypto, req, OWN_EMAIL, today(), &mut v);
        let errors = match v.finish() {
            Ok(()) => serde_json::Value::Null,
            Err(ApiError::Validation(errors)) => serde_json::to_value(&errors).unwrap(),
            Err(e) => panic!("unexpected {e}"),
        };
        (bracket, minor, errors)
    }

    fn years_ago(years: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026 - years, 10, 7).unwrap()
    }

    #[test]
    fn candidate_must_declare_the_age() {
        let (_, minor, errors) = age_of(&form(Role::Candidate, None, None));
        assert!(minor.is_none());
        assert_eq!(
            errors,
            serde_json::json!({"age": ["Укажите, есть ли вам 18 лет"]})
        );
    }

    #[test]
    fn employer_is_an_adult_whatever_the_form_says() {
        let req = form(
            Role::Employer,
            Some(AgeBracket::Minor),
            Some((years_ago(15), "o.petrova@example.ru")),
        );
        let (bracket, minor, errors) = age_of(&req);
        assert_eq!(bracket, AgeBracket::Adult);
        assert!(minor.is_none());
        assert_eq!(errors, serde_json::Value::Null);
        let (bracket, _, errors) = age_of(&form(Role::Employer, None, None));
        assert_eq!(bracket, AgeBracket::Adult);
        assert_eq!(errors, serde_json::Value::Null);
    }

    #[test]
    fn adult_candidate_gives_no_birth_date() {
        let req = form(
            Role::Candidate,
            Some(AgeBracket::Adult),
            Some((years_ago(15), "o.petrova@example.ru")),
        );
        let (bracket, minor, errors) = age_of(&req);
        assert_eq!(bracket, AgeBracket::Adult);
        assert!(minor.is_none(), "a minor block next to «18+» is ignored");
        assert_eq!(errors, serde_json::Value::Null);
    }

    #[test]
    fn minor_registration_is_open_from_14() {
        let req = form(
            Role::Candidate,
            Some(AgeBracket::Minor),
            Some((years_ago(13), "o.petrova@example.ru")),
        );
        assert_eq!(
            age_of(&req).2,
            serde_json::json!({"minor.birth_date": ["Регистрация открыта с 14 лет"]})
        );
    }

    #[test]
    fn minor_born_exactly_18_years_ago_is_sent_to_the_adult_form() {
        let req = form(
            Role::Candidate,
            Some(AgeBracket::Minor),
            Some((years_ago(18), "o.petrova@example.ru")),
        );
        assert_eq!(
            age_of(&req).2,
            serde_json::json!({"minor.birth_date": [
                "По дате рождения вам уже есть 18 — выберите «Мне есть 18 лет»"
            ]})
        );
    }

    #[test]
    fn guardian_address_must_not_be_the_minor_own() {
        let req = form(
            Role::Candidate,
            Some(AgeBracket::Minor),
            Some((years_ago(15), " Misha.Petrov@EXAMPLE.ru ")),
        );
        assert_eq!(
            age_of(&req).2,
            serde_json::json!({"minor.guardian.email": ["Нужна почта взрослого, а не ваша"]})
        );
    }

    #[test]
    fn minor_without_the_minor_block_is_refused() {
        let (bracket, minor, errors) =
            age_of(&form(Role::Candidate, Some(AgeBracket::Minor), None));
        assert_eq!(bracket, AgeBracket::Minor);
        assert!(minor.is_none());
        assert_eq!(
            errors,
            serde_json::json!({"minor": ["Укажите дату рождения и законного представителя"]})
        );
    }

    #[test]
    fn valid_minor_keeps_the_birth_date_and_a_lowercase_guardian_address() {
        let birth = NaiveDate::from_ymd_opt(2010, 3, 14).unwrap();
        let req = form(
            Role::Candidate,
            Some(AgeBracket::Minor),
            Some((birth, " O.Petrova@Example.RU ")),
        );
        let (bracket, minor, errors) = age_of(&req);
        assert_eq!(errors, serde_json::Value::Null);
        assert_eq!(bracket, AgeBracket::Minor);
        let (birth_date, contact) = minor.unwrap();
        assert_eq!(birth_date, birth);
        assert_eq!(contact.email, "o.petrova@example.ru");
        assert_eq!(contact.full_name, "Петрова Ольга Викторовна");
        assert_eq!(
            contact.bidx,
            Crypto::new(&[1; 32], &[2; 32]).blind_index("o.petrova@example.ru")
        );
    }

    fn document(
        title: &str,
        required_for: &[db::types::UserRole],
    ) -> db::consents::ConsentDocument {
        db::consents::ConsentDocument {
            id: Uuid::now_v7(),
            kind: title.to_lowercase(),
            version: 1,
            title: title.into(),
            body: String::new(),
            required_for: required_for.to_vec(),
            applies_to: required_for.to_vec(),
            published_at: "2026-01-01T00:00:00Z".parse().unwrap(),
        }
    }

    #[test]
    fn registration_lists_every_required_consent_not_accepted() {
        use db::types::UserRole::{Candidate, Employer};
        let docs = [
            document("Правила платформы", &[Candidate, Employer]),
            document("Обработка ПДн", &[Candidate]),
            document("Публикация профиля", &[Candidate]),
            document("Видеозапись теста", &[]),
        ];
        let err = require_consents(&docs, Candidate, &[docs[1].id]).unwrap_err();
        assert_eq!(err.summary(), (422, "validation_failed", vec!["consents"]));
        let ApiError::Validation(errors) = err else {
            unreachable!()
        };
        assert_eq!(
            serde_json::to_value(&errors).unwrap(),
            serde_json::json!({"consents": ["Нужно согласие: Правила платформы, Публикация профиля"]})
        );
        assert!(require_consents(&docs, Candidate, &[docs[0].id, docs[1].id, docs[2].id]).is_ok());
        assert!(
            require_consents(&docs, Employer, &[docs[0].id]).is_ok(),
            "an optional document is never required"
        );
        assert!(require_consents(&docs, Employer, &[docs[3].id]).is_err());
    }
}
