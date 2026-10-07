use axum::{
    extract::{DefaultBodyLimit, State},
    http::header,
    response::IntoResponse,
};
use catalog::{Category, Industry, WorkFormat};
use chrono::{DateTime, Utc};
use matching::ParsedResume;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::CandidateUser,
    contacts::{AccessChannel, ContactAccessEntry, ContactBasis, Contacts},
    dto::{SearchStatus, parse_slugs, slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Json, Multipart},
    pdf, pool,
    resume::{self, ResumeError},
    skills,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_profile, put_profile))
        .routes(routes!(get_privacy, put_privacy))
        .routes(routes!(contact_access))
        .routes(routes!(profile_pdf))
        .merge(
            OpenApiRouter::new()
                .routes(routes!(upload_resume))
                .layer(DefaultBodyLimit::max(resume::MAX_PDF_BYTES + 64 * 1024)),
        )
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExperienceEntry {
    pub company: String,
    pub position: String,
    /// `YYYY-MM`
    pub start: String,
    /// `YYYY-MM`, empty for the current job.
    pub end: Option<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileInput {
    pub full_name: String,
    pub contacts: Contacts,
    pub city: Option<String>,
    #[serde(default)]
    pub relocation: bool,
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    pub salary_expectation_min: Option<i64>,
    #[serde(default)]
    pub about: String,
    pub experience_years: Option<f32>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub soft_skills: Vec<String>,
    #[serde(default)]
    pub industries: Vec<Industry>,
    #[serde(default)]
    pub experience: Vec<ExperienceEntry>,
    /// Self-declared skill slugs. Test- and FSP-confirmed skills are added by the system.
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    Declared,
    Resume,
    Test,
    Fsp,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileSkill {
    pub slug: String,
    pub name: String,
    pub sources: Vec<SkillSource>,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub public_code: String,
    #[serde(flatten)]
    pub input: ProfileInput,
    pub skill_details: Vec<ProfileSkill>,
    pub category: Option<Category>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) fn parse_source(s: &str) -> Option<SkillSource> {
    Some(match s {
        "declared" => SkillSource::Declared,
        "resume" => SkillSource::Resume,
        "test" => SkillSource::Test,
        "fsp" => SkillSource::Fsp,
        _ => return None,
    })
}

pub(crate) async fn load_profile(
    state: &AppState,
    user: &CandidateUser,
) -> ApiResult<ProfileResponse> {
    load_profile_of(state, user.id).await
}

/// Callers other than the candidate must hold a `ContactGrant` for `candidate_id`.
pub(crate) async fn load_profile_of(
    state: &AppState,
    candidate_id: Uuid,
) -> ApiResult<ProfileResponse> {
    let c = db::candidates::candidate_profile(&state.pool, candidate_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let skill_rows = db::candidates::candidate_skills(&state.pool, candidate_id).await?;

    let mut skill_details: Vec<ProfileSkill> = Vec::new();
    for row in skill_rows {
        let Some(source) = parse_source(&row.source) else {
            continue;
        };
        match skill_details.last_mut() {
            Some(last) if last.slug == row.skill => last.sources.push(source),
            _ => skill_details.push(ProfileSkill {
                name: skills::display_name(&row.skill).to_owned(),
                slug: row.skill,
                sources: vec![source],
            }),
        }
    }
    let declared = skill_details
        .iter()
        .filter(|s| s.sources.contains(&SkillSource::Declared))
        .map(|s| s.slug.clone())
        .collect();

    let category = match (c.specialization.as_deref(), c.grade.as_deref()) {
        (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
        _ => None,
    };
    Ok(ProfileResponse {
        public_code: c.public_code,
        input: ProfileInput {
            full_name: c
                .full_name_enc
                .as_deref()
                .map(|b| state.crypto.decrypt_str(b))
                .transpose()?
                .unwrap_or_default(),
            contacts: c
                .contacts_enc
                .as_deref()
                .map(|b| state.crypto.decrypt_json(b))
                .transpose()?
                .unwrap_or_default(),
            city: c.city,
            relocation: c.relocation,
            work_formats: parse_slugs(&c.work_formats),
            salary_expectation_min: c.salary_expectation_min,
            about: c.about,
            experience_years: c.experience_years,
            roles: c.roles,
            soft_skills: c.soft_skills,
            industries: parse_slugs(&c.industries),
            experience: serde_json::from_value(c.experience)?,
            skills: declared,
        },
        skill_details,
        category,
        category_assigned_at: c.category_assigned_at,
        updated_at: c.updated_at,
    })
}

/// Own profile (contacts are stored encrypted and never shown to employers without a grant)
#[utoipa::path(get, path = "/profile", tag = "candidate",
    responses((status = 200, body = ProfileResponse), (status = 403, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_profile(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<ProfileResponse>> {
    Ok(Json(load_profile(&state, &user).await?))
}

fn non_empty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// As people type it: an optional leading `+`, 10–15 digits, spaces, brackets and dashes.
fn valid_phone(p: &str) -> bool {
    let body = p.strip_prefix('+').unwrap_or(p);
    let digits = body.chars().filter(char::is_ascii_digit).count();
    (10..=15).contains(&digits)
        && body
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '(' | ')' | '-'))
}

/// `@name`, `name` or a `t.me/name` link; Telegram names are 5–32 Latin letters, digits and `_`.
fn valid_telegram(t: &str) -> bool {
    let name = t
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("t.me/")
        .trim_start_matches('@');
    (5..=32).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn valid_month(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 7
        && b[4] == b'-'
        && s[..4].parse::<u16>().is_ok()
        && s[5..].parse::<u8>().is_ok_and(|m| (1..=12).contains(&m))
}

/// Save the profile (form or confirmed resume recognition result)
#[utoipa::path(put, path = "/profile", tag = "candidate",
    request_body = ProfileInput,
    responses((status = 200, body = ProfileResponse), (status = 422, description = "`validation_failed`: `full_name`, `contacts` (none given), `contacts.phone` (10–15 digits), `contacts.telegram` (5–32 Latin letters, digits, `_`), `salary_expectation_min`, `experience_years`, `about`, `skills`, `experience`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_profile(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(p): Json<ProfileInput>,
) -> ApiResult<Json<ProfileResponse>> {
    let mut v = Validator::default();
    let full_name = p.full_name.trim();
    let contacts = Contacts {
        phone: non_empty(p.contacts.phone),
        telegram: non_empty(p.contacts.telegram),
        email: non_empty(p.contacts.email),
    };
    let declared = skills::canonicalize(&p.skills);
    if let Err(unknown) = &declared {
        v.check_with(false, "skills", || {
            skills::unknown_message(unknown).into_owned()
        });
    }
    v.check(!full_name.is_empty(), "full_name", "Укажите ФИО")
        .check(
            full_name.chars().count() <= 200,
            "full_name",
            "Слишком длинное ФИО",
        )
        .check(
            contacts.phone.is_some() || contacts.telegram.is_some() || contacts.email.is_some(),
            "contacts",
            "Нужен хотя бы один способ связи",
        )
        .check(
            contacts.phone.as_deref().is_none_or(valid_phone),
            "contacts.phone",
            "Телефон — 10–15 цифр, например +7 999 555-44-33",
        )
        .check(
            contacts.telegram.as_deref().is_none_or(valid_telegram),
            "contacts.telegram",
            "Ник в Telegram — 5–32 латинские буквы, цифры или _, например @anna_dev",
        )
        .check(
            p.salary_expectation_min
                .is_none_or(|s| s > 0 && s <= 100_000_000),
            "salary_expectation_min",
            "Проверьте сумму",
        )
        .check(
            p.experience_years.is_none_or(|y| (0.0..=60.0).contains(&y)),
            "experience_years",
            "От 0 до 60 лет",
        )
        .check(
            p.about.chars().count() <= 5000,
            "about",
            "Не длиннее 5000 символов",
        )
        .check(p.skills.len() <= 60, "skills", "Не больше 60 навыков")
        .check(
            p.experience
                .iter()
                .all(|e| valid_month(&e.start) && e.end.as_deref().is_none_or(valid_month)),
            "experience",
            "Даты в формате ГГГГ-ММ",
        )
        .check(
            p.experience.len() <= 30,
            "experience",
            "Не больше 30 мест работы",
        );
    v.finish()?;

    let mut tx = state.pool.begin().await?;
    db::candidates::update_candidate_profile(
        &mut *tx,
        user.id,
        &db::candidates::ProfileUpdate {
            full_name_enc: Some(&state.crypto.encrypt_str(full_name)?),
            contacts_enc: Some(&state.crypto.encrypt_json(&contacts)?),
            city: non_empty(p.city).as_deref(),
            relocation: p.relocation,
            work_formats: &slugs(&p.work_formats),
            salary_expectation_min: p.salary_expectation_min,
            about: p.about.trim(),
            experience_years: p.experience_years,
            roles: &p.roles,
            soft_skills: &p.soft_skills,
            industries: &slugs(&p.industries),
            experience: &serde_json::to_value(&p.experience)?,
        },
    )
    .await?;
    db::candidates::delete_candidate_skills_by_source(&mut *tx, user.id, "declared").await?;
    for skill in declared.unwrap_or_default() {
        db::candidates::insert_candidate_skill(&mut *tx, user.id, &skill, "declared").await?;
    }
    tx.commit().await?;
    state.candidates.invalidate().await;
    Ok(Json(load_profile(&state, &user).await?))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Privacy {
    /// Shown in employer search at all.
    pub visible: bool,
    /// Full name on the anonymous card (contacts still need a grant).
    pub show_name: bool,
    pub show_fsp: bool,
    /// Company names in work history.
    pub show_experience: bool,
    pub search_status: SearchStatus,
}

/// Privacy settings: what employers see
#[utoipa::path(get, path = "/privacy", tag = "candidate",
    responses((status = 200, body = Privacy)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_privacy(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Privacy>> {
    let c = db::candidates::candidate_profile(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(Privacy {
        visible: c.visible,
        show_name: c.show_name,
        show_fsp: c.show_fsp,
        show_experience: c.show_experience,
        search_status: c.search_status.into(),
    }))
}

/// Showing the profile needs the 152-ФЗ consents first (`missing` comes ordered by kind, so
/// processing is named before publication), then, under 18, the representative's; hiding it
/// needs nothing.
fn visibility_gate(
    visible: bool,
    missing: Vec<db::consents::MissingConsent>,
    guardian_missing: Option<Uuid>,
) -> ApiResult<()> {
    if !visible {
        return Ok(());
    }
    if let Some(doc) = missing.into_iter().next() {
        return Err(ApiError::ConsentRequired {
            document_id: doc.id,
            kind: doc.kind,
            title: doc.title,
        });
    }
    match guardian_missing {
        Some(document_id) => Err(ApiError::GuardianConsentRequired { document_id }),
        None => Ok(()),
    }
}

/// Update privacy settings. Showing the profile in search needs active consents to processing
/// and to publication: without them the answer is 409 `consent_required` with the document to grant.
/// Under 18 it also needs the legal representative's consent (409 `guardian_consent_required`)
#[utoipa::path(put, path = "/privacy", tag = "candidate",
    request_body = Privacy,
    responses(
        (status = 200, body = Privacy),
        (status = 409, description = "`consent_required`: `visible: true` without an active `pd_processing` or `pd_publication` grant; `document_id` is the current version to grant via `POST /me/consents`. `guardian_consent_required`: under 18 without an active `guardian_consent` from the legal representative", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_privacy(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(p): Json<Privacy>,
) -> ApiResult<Json<Privacy>> {
    let missing = if p.visible {
        let required = [
            db::consents::PROCESSING.to_owned(),
            db::consents::PUBLICATION.to_owned(),
        ];
        db::consents::missing_consents(&state.pool, user.id, &required).await?
    } else {
        Vec::new()
    };
    let guardian_missing = if p.visible && missing.is_empty() {
        db::guardians::missing_guardian_consent(&state.pool, user.id)
            .await?
            .map(|doc| doc.id)
    } else {
        None
    };
    visibility_gate(p.visible, missing, guardian_missing)?;
    let status: db::types::SearchStatus = p.search_status.into();
    db::candidates::update_candidate_privacy(
        &state.pool,
        p.visible,
        p.show_name,
        p.show_fsp,
        p.show_experience,
        status,
        user.id,
    )
    .await?;
    state.candidates.invalidate().await;
    Ok(Json(p))
}

/// A journal is short for one person; the cap only bounds a pathological case.
const ACCESS_LOG_LIMIT: i64 = 500;

pub(crate) async fn access_log(
    pool: &sqlx::PgPool,
    candidate: Uuid,
) -> ApiResult<Vec<ContactAccessEntry>> {
    Ok(
        db::contacts::candidate_access_log(pool, candidate, ACCESS_LOG_LIMIT)
            .await?
            .into_iter()
            .filter_map(access_entry)
            .collect(),
    )
}

/// A row with a basis or channel this build does not know is left out rather than mislabelled.
fn access_entry(r: db::contacts::AccessRecord) -> Option<ContactAccessEntry> {
    Some(ContactAccessEntry {
        company_id: r.company_id,
        company_name: r.company_name,
        basis: ContactBasis::from_slug(&r.basis)?,
        channel: AccessChannel::from_slug(&r.channel)?,
        accessed_at: r.accessed_at,
    })
}

/// Who opened your contacts: company, time and basis — an accepted invitation or your own
/// application (ст. 14 152-ФЗ). Newest first, the latest 500 entries
#[utoipa::path(get, path = "/contact-access", tag = "candidate",
    responses((status = 200, body = Vec<ContactAccessEntry>), (status = 401, body = Problem), (status = 403, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn contact_access(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<ContactAccessEntry>>> {
    Ok(Json(access_log(&state.pool, user.id).await?))
}

#[derive(ToSchema)]
#[expect(
    dead_code,
    reason = "never constructed: an OpenAPI schema of the multipart body"
)]
pub struct ResumeUpload {
    /// PDF up to 10 MB with a text layer.
    #[schema(format = Binary, value_type = String)]
    file: Vec<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct ResumeDraft {
    /// Recognized values to prefill the profile form; nothing is saved to the profile until
    /// the candidate confirms it with `PUT /candidate/profile`.
    pub parsed: ParsedResume,
    pub recognized_fields: Vec<&'static str>,
}

fn recognized(p: &ParsedResume) -> Vec<&'static str> {
    let checks: [(&'static str, bool); 12] = [
        ("full_name", p.full_name.is_some()),
        ("email", p.email.is_some()),
        ("phone", p.phone.is_some()),
        ("telegram", p.telegram.is_some()),
        ("city", p.city.is_some()),
        ("skills", !p.skills.is_empty()),
        ("grade", p.grade_hint.is_some()),
        ("specialization", p.specialization_hint.is_some()),
        ("experience_years", p.experience_years.is_some()),
        ("roles", !p.roles.is_empty()),
        ("soft_skills", !p.soft_skills.is_empty()),
        ("industries", !p.industries.is_empty()),
    ];
    checks
        .into_iter()
        .filter_map(|(name, ok)| ok.then_some(name))
        .collect()
}

/// Upload a PDF resume: recognizes name, contacts, stack, grade, roles, experience, soft skills
#[utoipa::path(post, path = "/resume", tag = "candidate",
    request_body(content = ResumeUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = ResumeDraft),
        (status = 422, description = "`not_pdf`, `pdf_unreadable`, `pdf_no_text`, `pdf_too_large`; `validation_failed`: `file`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn upload_resume(
    State(state): State<AppState>,
    user: CandidateUser,
    Multipart(mut form): Multipart,
) -> ApiResult<Json<ResumeDraft>> {
    let mut file: Option<(String, Vec<u8>)> = None;
    while let Some(field) = form
        .next_field()
        .await
        .map_err(|_| ApiError::field("file", "Не удалось прочитать файл"))?
    {
        if field.name() == Some("file") {
            let name = field
                .file_name()
                .unwrap_or("resume.pdf")
                .chars()
                .take(200)
                .collect();
            let bytes = field
                .bytes()
                .await
                .map_err(|_| ApiError::field("file", "Файл слишком большой"))?;
            file = Some((name, bytes.to_vec()));
        }
    }
    let (filename, bytes) = file.ok_or_else(|| ApiError::field("file", "Приложите PDF"))?;
    let size = bytes.len();
    let text = resume::extract_text(bytes).await.map_err(|e| {
        let (code, msg) = match e {
            ResumeError::NotPdf => ("not_pdf", "Нужен файл PDF"),
            ResumeError::TooLarge => ("pdf_too_large", "PDF больше 10 МБ"),
            ResumeError::Unreadable => ("pdf_unreadable", "Не удалось прочитать PDF"),
            ResumeError::NoText => (
                "pdf_no_text",
                "В PDF нет текстового слоя — заполните профиль вручную",
            ),
        };
        ApiError::new(axum::http::StatusCode::UNPROCESSABLE_ENTITY, code, msg)
    })?;
    let parsed = matching::parse_resume(&text, Utc::now().timestamp());
    let fields = recognized(&parsed);

    let field_names: Vec<String> = fields.iter().map(|f| (*f).to_owned()).collect();
    let mut tx = state.pool.begin().await?;
    db::candidates::insert_resume_upload(
        &mut *tx,
        Uuid::now_v7(),
        user.id,
        &filename,
        size as i32,
        &field_names,
    )
    .await?;
    db::candidates::delete_candidate_skills_by_source(&mut *tx, user.id, "resume").await?;
    for skill in &parsed.skills {
        db::candidates::insert_candidate_skill(&mut *tx, user.id, skill, "resume").await?;
    }
    tx.commit().await?;
    state.candidates.invalidate().await;
    Ok(Json(ResumeDraft {
        parsed,
        recognized_fields: fields,
    }))
}

fn money(v: i64) -> String {
    let digits = v.to_string();
    let mut out = String::with_capacity(digits.len() + 4);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(ch);
    }
    out + "\u{a0}₽"
}

/// Standardized PDF profile (C-2)
#[utoipa::path(get, path = "/profile.pdf", tag = "candidate",
    responses((status = 200, content_type = "application/pdf", body = Vec<u8>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn profile_pdf(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<impl IntoResponse> {
    let (code, bytes) = profile_pdf_bytes(&state, user.id).await?;
    Ok(pdf_response(&code, bytes))
}

pub(crate) fn pdf_response(public_code: &str, bytes: Vec<u8>) -> impl IntoResponse + use<> {
    (
        [
            (header::CONTENT_TYPE, "application/pdf".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"profile-{public_code}.pdf\""),
            ),
        ],
        bytes,
    )
}

/// The standardized profile of `candidate_id`; callers other than the candidate must hold a
/// `ContactGrant` for them. Returns the anonymous code (for the file name) and the PDF.
pub(crate) async fn profile_pdf_bytes(
    state: &AppState,
    candidate_id: Uuid,
) -> ApiResult<(String, Vec<u8>)> {
    use pdf::{Line, Section};
    let p = load_profile_of(state, candidate_id).await?;
    let now = Utc::now();
    let mut sections = Vec::new();

    let c = &p.input.contacts;
    let contacts: Vec<Line> = [
        ("Телефон", &c.phone),
        ("Telegram", &c.telegram),
        ("Email", &c.email),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.as_ref().map(|v| Line::KeyValue(k.into(), v.clone())))
    .collect();
    if !contacts.is_empty() {
        sections.push(Section {
            heading: "Контакты".into(),
            lines: contacts,
        });
    }

    let mut key = Vec::new();
    if let Some(city) = &p.input.city {
        key.push(Line::KeyValue(
            "Город".into(),
            format!(
                "{city}{}",
                if p.input.relocation {
                    ", готов к переезду"
                } else {
                    ""
                }
            ),
        ));
    }
    if !p.input.work_formats.is_empty() {
        key.push(Line::KeyValue(
            "Формат".into(),
            p.input
                .work_formats
                .iter()
                .map(|f| f.name_ru())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if let Some(s) = p.input.salary_expectation_min {
        key.push(Line::KeyValue(
            "Ожидания".into(),
            format!("от {}", money(s)),
        ));
    }
    if let Some(y) = p.input.experience_years {
        key.push(Line::KeyValue("Опыт".into(), format!("{y:.1} лет")));
    }
    if !p.input.roles.is_empty() {
        key.push(Line::KeyValue("Роли".into(), p.input.roles.join(", ")));
    }
    if !p.input.industries.is_empty() {
        key.push(Line::KeyValue(
            "Отрасли".into(),
            p.input
                .industries
                .iter()
                .map(|i| i.name_ru())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if !key.is_empty() {
        sections.push(Section {
            heading: "Основное".into(),
            lines: key,
        });
    }

    let verified: Vec<&str> = p
        .skill_details
        .iter()
        .filter(|s| {
            s.sources
                .iter()
                .any(|x| matches!(x, SkillSource::Test | SkillSource::Fsp))
        })
        .map(|s| s.name.as_str())
        .collect();
    let other: Vec<&str> = p
        .skill_details
        .iter()
        .filter(|s| {
            !s.sources
                .iter()
                .any(|x| matches!(x, SkillSource::Test | SkillSource::Fsp))
        })
        .map(|s| s.name.as_str())
        .collect();
    let mut skill_lines = Vec::new();
    if !verified.is_empty() {
        skill_lines.push(Line::KeyValue("Подтверждены".into(), verified.join(", ")));
    }
    if !other.is_empty() {
        skill_lines.push(Line::KeyValue("Заявлены".into(), other.join(", ")));
    }
    if !skill_lines.is_empty() {
        sections.push(Section {
            heading: "Навыки".into(),
            lines: skill_lines,
        });
    }

    let fsp_lines = match db::fsp::fsp_link(&state.pool, candidate_id).await? {
        Some(link) => {
            let list = pool::achievements(Some(&link.achievements));
            let mut lines = vec![Line::KeyValue("ФСП ID".into(), link.fsp_id)];
            if list.is_empty() {
                lines.push(Line::Text("Нет данных ФСП".into()));
            }
            lines.extend(list.iter().take(12).map(|a| Line::Bullet(a.describe_ru())));
            lines
        }
        None => vec![Line::Text("Нет данных ФСП".into())],
    };
    sections.push(Section {
        heading: "Достижения ФСП".into(),
        lines: fsp_lines,
    });

    if !p.input.experience.is_empty() {
        let mut lines = Vec::new();
        for e in &p.input.experience {
            let end = e.end.as_deref().unwrap_or("по н. в.");
            lines.push(Line::KeyValue(
                format!("{} — {end}", e.start),
                format!("{}, {}", e.position, e.company),
            ));
            if !e.description.is_empty() {
                lines.push(Line::Bullet(e.description.clone()));
            }
        }
        sections.push(Section {
            heading: "Опыт работы".into(),
            lines,
        });
    }
    if !p.input.about.is_empty() {
        sections.push(Section {
            heading: "О себе".into(),
            lines: vec![Line::Text(p.input.about.clone())],
        });
    }

    let doc = pdf::ProfileDoc {
        title: if p.input.full_name.is_empty() {
            p.public_code.clone()
        } else {
            p.input.full_name.clone()
        },
        subtitle: p.category.map_or_else(
            || "Категория пока не подтверждена тестом".into(),
            Category::name_ru,
        ),
        badge: p.category_assigned_at.map(|at| {
            format!(
                "Категория подтверждена адаптивным тестом {}",
                at.format("%d.%m.%Y")
            )
        }),
        sections,
        footer: format!(
            "Сформировано платформой «ФСП Таланты» {}. Категория определяется только по результатам тестирования, не по резюме.",
            now.format("%d.%m.%Y")
        ),
    };
    let bytes = tokio::task::spawn_blocking(move || pdf::render(&doc)).await??;
    Ok((p.public_code, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contacts_look_like_contacts() {
        for ok in ["+7 999 555-44-33", "8 (999) 555-44-33", "+79995554433"] {
            assert!(valid_phone(ok), "{ok}");
        }
        for bad in [
            "+79995554433+79995554433",
            "555-44-33",
            "+7 999 abc",
            "7+9995554433",
        ] {
            assert!(!valid_phone(bad), "{bad}");
        }
        for ok in [
            "@anna_dev",
            "anna_dev",
            "https://t.me/anna_dev",
            "t.me/anna_dev",
        ] {
            assert!(valid_telegram(ok), "{ok}");
        }
        for bad in ["@ann", "@анна_dev", "@anna dev", "@anna-dev"] {
            assert!(!valid_telegram(bad), "{bad}");
        }
    }

    fn missing(kind: &str) -> db::consents::MissingConsent {
        db::consents::MissingConsent {
            id: Uuid::now_v7(),
            kind: kind.into(),
            title: format!("Согласие {kind}"),
        }
    }

    #[test]
    fn hiding_the_profile_needs_no_consent() {
        let guardian = Some(Uuid::now_v7());
        assert!(visibility_gate(false, vec![missing(db::consents::PROCESSING)], guardian).is_ok());
        assert!(visibility_gate(true, Vec::new(), None).is_ok());
    }

    #[test]
    fn showing_the_profile_names_the_first_missing_consent() {
        let processing = missing(db::consents::PROCESSING);
        let id = processing.id;
        let err = visibility_gate(
            true,
            vec![processing, missing(db::consents::PUBLICATION)],
            Some(Uuid::now_v7()),
        )
        .unwrap_err();
        let ApiError::ConsentRequired {
            document_id, kind, ..
        } = err
        else {
            panic!("expected consent_required")
        };
        assert_eq!((document_id, kind.as_str()), (id, db::consents::PROCESSING));

        let publication = missing(db::consents::PUBLICATION);
        let id = publication.id;
        let err = visibility_gate(true, vec![publication], None).unwrap_err();
        assert!(
            matches!(err, ApiError::ConsentRequired { document_id, ref kind, .. }
                if document_id == id && kind == db::consents::PUBLICATION)
        );
    }

    #[test]
    fn minor_with_both_consents_waits_for_the_representative() {
        let doc = Uuid::now_v7();
        let err = visibility_gate(true, Vec::new(), Some(doc)).unwrap_err();
        assert_eq!(err.summary(), (409, "guardian_consent_required", vec![]));
        assert!(
            matches!(err, ApiError::GuardianConsentRequired { document_id } if document_id == doc)
        );
    }

    fn record(
        basis: &str,
        channel: &str,
        company_name: Option<&str>,
    ) -> db::contacts::AccessRecord {
        db::contacts::AccessRecord {
            company_id: Uuid::now_v7(),
            company_name: company_name.map(str::to_owned),
            basis: basis.into(),
            channel: channel.into(),
            accessed_at: "2026-10-07T12:00:00Z".parse().unwrap(),
        }
    }

    #[test]
    fn access_log_entry_names_the_company_and_basis_only() {
        let entry = access_entry(record("invitation", "api", Some("ООО «Северный код»"))).unwrap();
        assert_eq!(entry.basis, ContactBasis::Invitation);
        assert_eq!(entry.channel, AccessChannel::Api);
        let json = serde_json::to_value(&entry).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "accessed_at",
                "basis",
                "channel",
                "company_id",
                "company_name"
            ]
        );
        assert_eq!(json["company_name"], "ООО «Северный код»");
    }

    #[test]
    fn access_log_keeps_entries_of_a_deleted_company() {
        let entry = access_entry(record("application", "cabinet", None)).unwrap();
        assert_eq!(entry.company_name, None);
        assert_eq!(entry.basis, ContactBasis::Application);
        assert!(serde_json::to_value(&entry).unwrap()["company_name"].is_null());
    }

    #[test]
    fn access_log_skips_an_unknown_basis_or_channel() {
        assert!(access_entry(record("referral", "cabinet", None)).is_none());
        assert!(access_entry(record("application", "fax", None)).is_none());
        assert!(access_entry(record("application", "webhook", None)).is_some());
    }
}
