//! Anonymous candidate card for employers. It has no contact fields by construction (V-2):
//! contacts are only reachable through `contacts::ContactGrant`.

use catalog::{Category, Industry, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{FspScore, SkillSource};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    crypto::Crypto,
    dto::{InvitationStatus, SearchStatus, parse_slugs},
    error::ApiResult,
    pool,
    proctoring::{self, ProctoringBadge},
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
    /// How the test behind the current category was proctored; `None` for tests taken without
    /// proctoring. Never carries the video.
    pub proctoring: Option<ProctoringBadge>,
    /// «Результат подтверждён»: a moderator reviewed the test behind the current category and
    /// confirmed it. The decision itself, the comment and the video stay with the moderator and
    /// the candidate.
    pub result_confirmed: bool,
    /// Under 18 (registered as 14–17, legal representative's consent given): show a neutral
    /// «14–17 лет». The birth date itself never reaches employers.
    pub minor: bool,
    pub relation: Relation,
}

/// A card with what decides whether it may be shown; the flags never reach the response.
pub struct Loaded {
    pub card: CandidateCard,
    /// Visible with the publication consents in force: the profile can be opened.
    pub published: bool,
    /// Published and not closed to offers: search and shortlists may list it.
    pub searchable: bool,
}

impl Loaded {
    /// An application or invitation keeps the candidate in the company's view.
    pub fn related(&self) -> bool {
        self.card.relation.applied || self.card.relation.invitation_status.is_some()
    }
}

/// Cards of `ids` in their order, one database round trip.
pub async fn load(
    pg: &PgPool,
    crypto: &Crypto,
    company_id: Uuid,
    ids: &[Uuid],
    now: i64,
) -> ApiResult<Vec<Loaded>> {
    let rows = db::pool::candidate_cards(pg, company_id, ids).await?;
    let mut cards = Vec::with_capacity(rows.len());
    for r in rows {
        let (published, searchable) = (r.published, r.searchable());
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

        let proctoring = r.proctoring().as_ref().map(proctoring::badge);
        let experience: Vec<ExperienceEntry> =
            serde_json::from_value(r.experience).unwrap_or_default();
        let invitation_status = r.invitation_status.map(InvitationStatus::from);
        cards.push(Loaded {
            published,
            searchable,
            card: CandidateCard {
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
                proctoring,
                result_confirmed: r.result_confirmed.unwrap_or(false),
                minor: r.minor,
                relation: Relation {
                    contacts_available: r.applied
                        || invitation_status == Some(InvitationStatus::Accepted),
                    invitation_status,
                    applied: r.applied,
                },
            },
        });
    }
    cards.sort_by_key(|c| {
        ids.iter()
            .position(|id| *id == c.card.id)
            .unwrap_or(usize::MAX)
    });
    Ok(cards)
}
