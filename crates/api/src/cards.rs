//! Anonymous candidate card for employers. It has no contact fields by construction (V-2):
//! contacts are only reachable through `contacts::ContactGrant`.
//!
//! The company-independent part of a card is kept in the pool snapshot of the candidate's
//! specialization (architecture.md §12.2) and lives exactly as long as it: every write that
//! changes a card bumps the cache generation. What must never lag — publication, search status,
//! the privacy flags — and what changes without a bump — activity time, age, the company's
//! relation — is read live on every display ([`db::pool::card_live`]).

use std::sync::Arc;

use catalog::{Category, Industry, Specialization, WorkFormat};
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
    proctoring::{self, ProctoringBadge},
    routes::candidate_profile::ExperienceEntry,
    skills,
    state::AppState,
};

#[derive(Clone, Serialize, ToSchema)]
pub struct CardSkill {
    pub slug: String,
    pub name: String,
    pub sources: Vec<SkillSource>,
    /// Confirmed by the test or the FSP registry.
    pub verified: bool,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct CardAchievement {
    pub text: String,
    pub date: String,
    pub protocol_url: Option<String>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct CardFsp {
    pub score: FspScore,
    pub achievements: Vec<CardAchievement>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct CardExperience {
    /// Hidden unless the candidate allows showing company names.
    pub company: Option<String>,
    pub position: String,
    pub start: String,
    pub end: Option<String>,
    pub description: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Relation {
    pub invitation_status: Option<InvitationStatus>,
    pub applied: bool,
    /// Accepted invitation or own application: `/employer/candidates/{id}/contacts` will answer.
    pub contacts_available: bool,
}

#[derive(Clone, Serialize, ToSchema)]
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

/// The privacy flags a card was built with. A cached card serves only while the live flags are
/// the same, so a lagging replica never shows a name, FSP data or a company the candidate has
/// just hidden.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Shown {
    name: bool,
    fsp: bool,
    experience: bool,
}

/// A card as cached in the pool snapshot; its live fields are overwritten on every display.
pub struct CachedCard {
    card: CandidateCard,
    shown: Shown,
}

fn relation(invitation: Option<db::types::InvitationStatus>, applied: bool) -> Relation {
    let invitation_status = invitation.map(InvitationStatus::from);
    Relation {
        contacts_available: applied || invitation_status == Some(InvitationStatus::Accepted),
        invitation_status,
        applied,
    }
}

impl CachedCard {
    fn show(&self, live: db::pool::CardLive) -> Loaded {
        let mut card = self.card.clone();
        card.search_status = live.search_status.into();
        card.last_activity_at = live.last_activity_at;
        card.minor = live.minor;
        card.relation = relation(live.invitation_status, live.applied);
        Loaded {
            published: live.published,
            searchable: live.searchable(),
            card,
        }
    }
}

/// Cards of `ids` in their order: cached ones with one light query of their live columns, the
/// rest with one full query, which also fills the cache for candidates of the current snapshots.
pub async fn load(
    state: &AppState,
    company_id: Uuid,
    ids: &[Uuid],
    now: i64,
) -> ApiResult<Vec<Loaded>> {
    let pools = state.candidates.current_pools().await;
    let mut cached: Vec<(Uuid, Arc<CachedCard>)> = Vec::new();
    let mut fetch: Vec<Uuid> = Vec::new();
    for &id in ids {
        match pools.iter().flatten().find_map(|p| p.value.card(id)) {
            Some(card) => cached.push((id, card)),
            None => fetch.push(id),
        }
    }
    let mut cards = Vec::with_capacity(ids.len());
    if !cached.is_empty() {
        let hit_ids: Vec<Uuid> = cached.iter().map(|(id, _)| *id).collect();
        for live in db::pool::card_live(&state.pool, company_id, &hit_ids).await? {
            let Some((_, card)) = cached.iter().find(|(id, _)| *id == live.user_id) else {
                continue;
            };
            let shown = Shown {
                name: live.show_name,
                fsp: live.show_fsp,
                experience: live.show_experience,
            };
            if card.shown == shown {
                cards.push(card.show(live));
            } else {
                fetch.push(live.user_id);
            }
        }
    }
    let hits = cards.len();
    state.candidates.card_lookups.record(hits, fetch.len());
    if !fetch.is_empty() {
        for r in db::pool::candidate_cards(&state.pool, company_id, &fetch).await? {
            let (loaded, shown, specialization) = build(r, &state.crypto, now)?;
            if let Some(snapshot) = specialization.and_then(|s| pools[s as usize].as_ref()) {
                snapshot.value.keep_card(loaded.card.id, || CachedCard {
                    card: loaded.card.clone(),
                    shown,
                });
            }
            cards.push(loaded);
        }
    }
    cards.sort_by_key(|c| {
        ids.iter()
            .position(|id| *id == c.card.id)
            .unwrap_or(usize::MAX)
    });
    Ok(cards)
}

fn build(
    r: db::pool::CandidateCard,
    crypto: &Crypto,
    now: i64,
) -> ApiResult<(Loaded, Shown, Option<Specialization>)> {
    let (published, searchable) = (r.published, r.searchable());
    let shown = Shown {
        name: r.show_name,
        fsp: r.show_fsp,
        experience: r.show_experience,
    };
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
    let experience: Vec<ExperienceEntry> = serde_json::from_value(r.experience).unwrap_or_default();
    let card = CandidateCard {
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
        minor: r.minor,
        relation: relation(r.invitation_status, r.applied),
    };
    Ok((
        Loaded {
            card,
            published,
            searchable,
        },
        shown,
        category.map(|c| c.specialization),
    ))
}
