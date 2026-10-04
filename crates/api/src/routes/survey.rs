use std::collections::HashMap;

use axum::{Json, extract::State, http::StatusCode};
use catalog::{Direction, Grade, Industry, Specialization};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::CandidateUser,
    dto::{parse_slugs, slugs},
    error::{ApiResult, Problem, Validator},
    skills,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(submit_survey))
        .routes(routes!(survey_history))
}

#[derive(Deserialize, ToSchema)]
pub struct SurveyInput {
    pub direction: Direction,
    pub specialization: Specialization,
    /// The grade the candidate expects to confirm with the test.
    pub claimed_grade: Grade,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub industries: Vec<Industry>,
    pub experience_years: Option<f32>,
    /// Self-assessment per topic, 1–5. Only a starting hint for the adaptive test, never a score.
    /// Keys come from the client, so the map keeps std's DoS-resistant hasher.
    #[serde(default)]
    pub self_assessment: HashMap<String, u8>,
}

#[derive(Serialize, ToSchema)]
pub struct SurveyResponse {
    pub id: Uuid,
    pub direction: Direction,
    pub specialization: Specialization,
    pub claimed_grade: Grade,
    pub skills: Vec<String>,
    pub industries: Vec<Industry>,
    pub experience_years: Option<f32>,
    pub self_assessment: HashMap<String, u8>,
    pub created_at: DateTime<Utc>,
}

/// Onboarding survey: direction, specialization, stack and the claimed grade
#[utoipa::path(post, path = "/survey", tag = "candidate",
    request_body = SurveyInput,
    responses((status = 201, body = SurveyResponse), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn submit_survey(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(s): Json<SurveyInput>,
) -> ApiResult<(StatusCode, Json<SurveyResponse>)> {
    let unknown: Vec<&str> = s
        .skills
        .iter()
        .map(String::as_str)
        .filter(|k| !skills::is_known(k))
        .collect();
    let mut v = Validator::default();
    v.check(
        s.specialization.direction() == s.direction,
        "specialization",
        "Специализация не из выбранного направления",
    )
    .check_with(unknown.is_empty(), "skills", || {
        format!("Неизвестные навыки: {}", unknown.join(", "))
    })
    .check(s.skills.len() <= 60, "skills", "Не больше 60 навыков")
    .check(
        s.experience_years.is_none_or(|y| (0.0..=60.0).contains(&y)),
        "experience_years",
        "От 0 до 60 лет",
    )
    .check(
        s.self_assessment.values().all(|x| (1..=5).contains(x)),
        "self_assessment",
        "Оценки от 1 до 5",
    )
    .check(
        s.self_assessment.len() <= 30,
        "self_assessment",
        "Слишком много тем",
    );
    v.finish()?;

    let id = Uuid::now_v7();
    let mut client = state.pool.get().await?;
    let tx = client.transaction().await?;
    db::queries::survey::insert_survey()
        .bind(
            &tx,
            &id,
            &user.id,
            &s.direction.slug(),
            &s.specialization.slug(),
            &s.claimed_grade.slug(),
            &s.skills,
            &slugs(&s.industries),
            &s.experience_years,
            &serde_json::to_value(&s.self_assessment)?,
        )
        .await?;
    for skill in &s.skills {
        db::queries::candidates::insert_candidate_skill()
            .bind(&tx, &user.id, skill, &"declared")
            .await?;
    }
    db::queries::candidates::touch_candidate_activity()
        .bind(&tx, &user.id)
        .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(SurveyResponse {
            id,
            direction: s.direction,
            specialization: s.specialization,
            claimed_grade: s.claimed_grade,
            skills: s.skills,
            industries: s.industries,
            experience_years: s.experience_years,
            self_assessment: s.self_assessment,
            created_at: Utc::now(),
        }),
    ))
}

/// Survey history (C-4)
#[utoipa::path(get, path = "/survey/history", tag = "candidate",
    responses((status = 200, body = Vec<SurveyResponse>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn survey_history(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<SurveyResponse>>> {
    let client = state.pool.get().await?;
    let rows = db::queries::survey::survey_history()
        .bind(&client, &user.id)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(SurveyResponse {
            id: r.id,
            direction: r.direction.parse()?,
            specialization: r.specialization.parse()?,
            claimed_grade: r.claimed_grade.parse()?,
            skills: r.skills,
            industries: parse_slugs(&r.industries),
            experience_years: r.experience_years,
            self_assessment: serde_json::from_value(r.self_assessment)?,
            created_at: r.created_at,
        });
    }
    Ok(Json(out))
}
