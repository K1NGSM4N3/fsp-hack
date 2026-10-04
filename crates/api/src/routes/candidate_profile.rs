use axum::{Json, extract::State};
use catalog::{Category, Industry, WorkFormat};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::CandidateUser,
    contacts::Contacts,
    dto::{SearchStatus, parse_slugs, slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    skills,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_profile, put_profile)).routes(routes!(get_privacy, put_privacy))
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

async fn load_profile(state: &AppState, user: &CandidateUser) -> ApiResult<ProfileResponse> {
    let client = state.pool.get().await?;
    let c = db::queries::candidates::candidate_profile().bind(&client, &user.id).one().await?;
    let skill_rows = db::queries::candidates::candidate_skills().bind(&client, &user.id).all().await?;

    let mut skill_details: Vec<ProfileSkill> = Vec::new();
    for row in skill_rows {
        let Some(source) = parse_source(&row.source) else { continue };
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
            full_name: c.full_name_enc.as_deref().map(|b| state.crypto.decrypt_str(b)).transpose()?.unwrap_or_default(),
            contacts: c.contacts_enc.as_deref().map(|b| state.crypto.decrypt_json(b)).transpose()?.unwrap_or_default(),
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
async fn get_profile(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Json<ProfileResponse>> {
    Ok(Json(load_profile(&state, &user).await?))
}

fn non_empty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

fn valid_month(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 7 && b[4] == b'-' && s[..4].parse::<u16>().is_ok() && s[5..].parse::<u8>().is_ok_and(|m| (1..=12).contains(&m))
}

/// Save the profile (form or confirmed resume recognition result)
#[utoipa::path(put, path = "/profile", tag = "candidate",
    request_body = ProfileInput,
    responses((status = 200, body = ProfileResponse), (status = 422, body = Problem)),
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
    let unknown: Vec<&str> = p.skills.iter().map(String::as_str).filter(|s| !skills::is_known(s)).collect();
    v.check(!full_name.is_empty(), "full_name", "Укажите ФИО")
        .check(full_name.len() <= 200, "full_name", "Слишком длинное ФИО")
        .check(
            contacts.phone.is_some() || contacts.telegram.is_some() || contacts.email.is_some(),
            "contacts",
            "Нужен хотя бы один способ связи",
        )
        .check(p.salary_expectation_min.is_none_or(|s| s > 0 && s <= 100_000_000), "salary_expectation_min", "Проверьте сумму")
        .check(p.experience_years.is_none_or(|y| (0.0..=60.0).contains(&y)), "experience_years", "От 0 до 60 лет")
        .check(p.about.len() <= 5000, "about", "Не длиннее 5000 символов")
        .check(p.skills.len() <= 60, "skills", "Не больше 60 навыков")
        .check_with(unknown.is_empty(), "skills", || format!("Неизвестные навыки: {}", unknown.join(", ")))
        .check(
            p.experience.iter().all(|e| valid_month(&e.start) && e.end.as_deref().is_none_or(valid_month)),
            "experience",
            "Даты в формате ГГГГ-ММ",
        )
        .check(p.experience.len() <= 30, "experience", "Не больше 30 мест работы");
    v.finish()?;

    let mut client = state.pool.get().await?;
    let tx = client.transaction().await?;
    db::queries::candidates::update_candidate_profile()
        .bind(
            &tx,
            &Some(state.crypto.encrypt_str(full_name)),
            &Some(state.crypto.encrypt_json(&contacts)?),
            &non_empty(p.city).as_deref(),
            &p.relocation,
            &slugs(&p.work_formats),
            &p.salary_expectation_min,
            &p.about.trim(),
            &p.experience_years,
            &p.roles,
            &p.soft_skills,
            &slugs(&p.industries),
            &serde_json::to_value(&p.experience)?,
            &user.id,
        )
        .await?;
    db::queries::candidates::delete_candidate_skills_by_source().bind(&tx, &user.id, &"declared").await?;
    for skill in &p.skills {
        db::queries::candidates::insert_candidate_skill().bind(&tx, &user.id, skill, &"declared").await?;
    }
    tx.commit().await?;
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
async fn get_privacy(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Json<Privacy>> {
    let client = state.pool.get().await?;
    let c = db::queries::candidates::candidate_profile().bind(&client, &user.id).opt().await?.ok_or_else(ApiError::not_found)?;
    Ok(Json(Privacy {
        visible: c.visible,
        show_name: c.show_name,
        show_fsp: c.show_fsp,
        show_experience: c.show_experience,
        search_status: c.search_status.into(),
    }))
}

/// Update privacy settings
#[utoipa::path(put, path = "/privacy", tag = "candidate",
    request_body = Privacy,
    responses((status = 200, body = Privacy)),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_privacy(State(state): State<AppState>, user: CandidateUser, Json(p): Json<Privacy>) -> ApiResult<Json<Privacy>> {
    let client = state.pool.get().await?;
    let status: db::types::SearchStatus = p.search_status.into();
    db::queries::candidates::update_candidate_privacy()
        .bind(&client, &p.visible, &p.show_name, &p.show_fsp, &p.show_experience, &status, &user.id)
        .await?;
    Ok(Json(p))
}
