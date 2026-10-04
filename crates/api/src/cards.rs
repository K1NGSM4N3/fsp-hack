//! Anonymous candidate card for employers. It has no contact fields by construction (V-2):
//! contacts are only reachable through `contacts::ContactGrant`.

use catalog::{Category, Industry, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{FspScore, SkillSource};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    crypto::Crypto,
    dto::{InvitationStatus, SearchStatus, parse_slugs},
    error::ApiResult,
    pool,
    routes::candidate_profile::ExperienceEntry,
    skills,
};

#[derive(Serialize, ToSchema)]
pub struct CardSkill {
    pub slug: String,
    pub name: String,
    pub sources: Vec<SkillSource>,
    /// Confirmed by the test or the FSP registry.
    pub verified: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CardAchievement {
    pub text: String,
    pub date: String,
    pub protocol_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CardFsp {
    pub score: FspScore,
    pub achievements: Vec<CardAchievement>,
}

#[derive(Serialize, ToSchema)]
pub struct CardExperience {
    /// Hidden unless the candidate allows showing company names.
    pub company: Option<String>,
    pub position: String,
    pub start: String,
    pub end: Option<String>,
    pub description: String,
}

#[derive(Serialize, ToSchema)]
pub struct Relation {
    pub invitation_status: Option<InvitationStatus>,
    pub applied: bool,
    /// Accepted invitation or own application: `/employer/candidates/{id}/contacts` will answer.
    pub contacts_available: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CandidateCard {
    pub id: Uuid,
    pub code: String,
    /// Only when the candidate chose to show it.
    pub display_name: Option<String>,
    pub category: Option<Category>,
    pub category_name: Option<String>,
    /// Position inside the grade band, 0..1 (comparable across different adaptive tests).
    pub test_strength: Option<f32>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub skills: Vec<CardSkill>,
    /// `None` when the candidate hides FSP data; `score.has_history = false` when there is none.
    pub fsp: Option<CardFsp>,
    pub experience_years: Option<f32>,
    pub roles: Vec<String>,
    pub soft_skills: Vec<String>,
    pub industries: Vec<Industry>,
    pub experience: Vec<CardExperience>,
    pub about: String,
    pub city: Option<String>,
    pub relocation: bool,
    pub work_formats: Vec<WorkFormat>,
    pub salary_expectation_min: Option<i64>,
    pub search_status: SearchStatus,
    pub last_activity_at: DateTime<Utc>,
    pub task_rating: Option<f32>,
    pub tasks_done: i64,
    pub relation: Relation,
}

pub async fn load(
    client: &deadpool_postgres::Object,
    crypto: &Crypto,
    company_id: Uuid,
    ids: &[Uuid],
    now: i64,
) -> ApiResult<Vec<CandidateCard>> {
    let rows = db::queries::pool::candidate_cards()
        .bind(client, &ids)
        .all()
        .await?;
    let relations = db::queries::pool::company_relations()
        .bind(client, &company_id, &ids)
        .all()
        .await?;
    let mut cards = Vec::with_capacity(rows.len());
    for r in rows {
        let category = match (r.specialization.as_deref(), r.grade.as_deref()) {
            (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
            _ => None,
        };
        let mut card_skills: Vec<CardSkill> = Vec::new();
        for (slug, src) in r.skills.iter().zip(&r.skill_sources) {
            let source = pool::source(src);
            match card_skills.last_mut() {
                Some(last) if &last.slug == slug => {
                    last.sources.push(source);
                    last.verified |= source.is_verified();
                }
                _ => card_skills.push(CardSkill {
                    slug: slug.clone(),
                    name: skills::display_name(slug).to_owned(),
                    sources: vec![source],
                    verified: source.is_verified(),
                }),
            }
        }
        card_skills.sort_by_key(|s| !s.verified);

        let fsp = match (r.show_fsp, category) {
            (true, Some(c)) => {
                let list = pool::achievements(r.achievements.as_ref());
                Some(CardFsp {
                    score: matching::fsp_score(&list, c.specialization, now),
                    achievements: list
                        .iter()
                        .map(|a| CardAchievement {
                            text: a.describe_ru(),
                            date: a.date.clone(),
                            protocol_url: a.protocol_url.clone(),
                        })
                        .collect(),
                })
            }
            _ => None,
        };

        let experience: Vec<ExperienceEntry> =
            serde_json::from_value(r.experience).unwrap_or_default();
        let relation = relations.iter().find(|x| x.candidate_id == r.user_id);
        let invitation_status = relation
            .and_then(|x| x.invitation_status)
            .map(InvitationStatus::from);
        let applied = relation.is_some_and(|x| x.applied);
        cards.push(CandidateCard {
            id: r.user_id,
            code: r.public_code,
            display_name: if r.show_name {
                r.full_name_enc
                    .as_deref()
                    .map(|b| crypto.decrypt_str(b))
                    .transpose()?
            } else {
                None
            },
            category_name: category.map(Category::name_ru),
            test_strength: match (category, r.theta) {
                (Some(c), Some(t)) => Some(matching::test_strength(c.grade, t as f32)),
                _ => None,
            },
            category,
            category_assigned_at: r.category_assigned_at,
            skills: card_skills,
            fsp,
            experience_years: r.experience_years,
            roles: r.roles,
            soft_skills: r.soft_skills,
            industries: parse_slugs(&r.industries),
            experience: experience
                .into_iter()
                .map(|e| CardExperience {
                    company: r.show_experience.then_some(e.company),
                    position: e.position,
                    start: e.start,
                    end: e.end,
                    description: e.description,
                })
                .collect(),
            about: r.about,
            city: r.city,
            relocation: r.relocation,
            work_formats: parse_slugs(&r.work_formats),
            salary_expectation_min: r.salary_expectation_min,
            search_status: r.search_status.into(),
            last_activity_at: r.last_activity_at,
            task_rating: r.task_rating,
            tasks_done: r.tasks_done,
            relation: Relation {
                contacts_available: applied
                    || invitation_status == Some(InvitationStatus::Accepted),
                invitation_status,
                applied,
            },
        });
    }
    cards.sort_by_key(|c| ids.iter().position(|id| *id == c.id).unwrap_or(usize::MAX));
    Ok(cards)
}
