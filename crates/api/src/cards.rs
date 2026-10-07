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

    /// The profile page opens for a published candidate, and for one the company is already in
    /// touch with even after they hid the profile.
    pub fn visible_to_company(&self) -> bool {
        self.published || self.related()
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
    fn reusable(&self, live: &db::pool::CardLive) -> bool {
        self.shown
            == Shown {
                name: live.show_name,
                fsp: live.show_fsp,
                experience: live.show_experience,
            }
    }

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
            if card.reusable(&live) {
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
        result_confirmed: r.result_confirmed.unwrap_or(false),
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

#[cfg(test)]
mod tests {
    use catalog::Grade;
    use db::types::{InvitationStatus as Db, SearchStatus as DbSearch};

    use super::*;

    const NOW: i64 = 1_791_374_400;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn crypto() -> Crypto {
        Crypto::new(&[1; 32], &[2; 32])
    }

    fn row(crypto: &Crypto) -> db::pool::CandidateCard {
        db::pool::CandidateCard {
            user_id: Uuid::from_u128(1),
            public_code: "C-0001".into(),
            full_name_enc: Some(crypto.encrypt_str("Анна Смирнова").unwrap()),
            show_name: true,
            show_fsp: true,
            show_experience: true,
            city: Some("Казань".into()),
            relocation: false,
            work_formats: vec!["remote".into()],
            salary_expectation_min: Some(150_000),
            about: "Пишу на Go".into(),
            experience_years: Some(3.0),
            roles: Vec::new(),
            soft_skills: Vec::new(),
            industries: Vec::new(),
            experience: serde_json::json!([{
                "company": "ООО «Северный код»",
                "position": "Backend-разработчик",
                "start": "2023-01",
                "end": null,
                "description": ""
            }]),
            search_status: DbSearch::Active,
            specialization: Some("backend".into()),
            grade: Some("middle".into()),
            theta: Some(0.4),
            category_assigned_at: Some(at("2026-09-01T10:00:00Z")),
            last_activity_at: at("2026-10-01T10:00:00Z"),
            achievements: None,
            skills: vec!["go".into(), "go".into(), "postgresql".into()],
            skill_sources: vec!["declared".into(), "test".into(), "declared".into()],
            task_rating: None,
            tasks_done: 0,
            minor: false,
            published: true,
            invitation_status: None,
            applied: false,
            result_confirmed: None,
            proctoring_camera: None,
            proctoring_chunks: None,
            tab_hidden: None,
            window_blur: None,
            fullscreen_exit: None,
            paste: None,
            camera_lost: None,
            recording_failed: None,
        }
    }

    fn live(r: &db::pool::CandidateCard) -> db::pool::CardLive {
        db::pool::CardLive {
            user_id: r.user_id,
            show_name: r.show_name,
            show_fsp: r.show_fsp,
            show_experience: r.show_experience,
            search_status: r.search_status,
            last_activity_at: r.last_activity_at,
            minor: r.minor,
            published: r.published,
            invitation_status: r.invitation_status,
            applied: r.applied,
        }
    }

    #[test]
    fn contacts_open_after_an_application_or_an_accepted_invitation() {
        assert!(!relation(None, false).contacts_available);
        assert!(relation(None, true).contacts_available);
        assert!(relation(Some(Db::Accepted), false).contacts_available);
        assert!(relation(Some(Db::Declined), true).contacts_available);
        for pending in [
            Db::Sent,
            Db::Viewed,
            Db::Declined,
            Db::Expired,
            Db::Withdrawn,
        ] {
            let r = relation(Some(pending), false);
            assert!(!r.contacts_available, "{pending:?}");
            assert_eq!(r.invitation_status, Some(pending.into()));
        }
    }

    #[test]
    fn card_follows_the_privacy_flags() {
        let crypto = crypto();
        let (loaded, ..) = build(row(&crypto), &crypto, NOW).unwrap();
        let card = loaded.card;
        assert_eq!(card.display_name.as_deref(), Some("Анна Смирнова"));
        assert_eq!(
            card.experience[0].company.as_deref(),
            Some("ООО «Северный код»")
        );
        assert!(card.fsp.is_some());
        assert_eq!(
            card.category,
            Some(Category::new(Specialization::Backend, Grade::Middle))
        );

        let hidden = db::pool::CandidateCard {
            show_name: false,
            show_experience: false,
            show_fsp: false,
            ..row(&crypto)
        };
        let (loaded, ..) = build(hidden, &crypto, NOW).unwrap();
        let card = loaded.card;
        assert_eq!(card.display_name, None);
        assert_eq!(card.experience.len(), 1);
        assert_eq!(card.experience[0].company, None);
        assert_eq!(card.experience[0].position, "Backend-разработчик");
        assert!(card.fsp.is_none());
    }

    #[test]
    fn one_skill_from_several_sources_is_one_verified_entry() {
        let crypto = crypto();
        let (loaded, ..) = build(row(&crypto), &crypto, NOW).unwrap();
        let skills: Vec<(&str, &[SkillSource], bool)> = loaded
            .card
            .skills
            .iter()
            .map(|s| (s.slug.as_str(), s.sources.as_slice(), s.verified))
            .collect();
        assert_eq!(
            skills,
            [
                ("go", &[SkillSource::Declared, SkillSource::Test][..], true),
                ("postgresql", &[SkillSource::Declared][..], false),
            ]
        );
    }

    #[test]
    fn minor_flag_and_visibility_pass_through() {
        let crypto = crypto();
        let minor = db::pool::CandidateCard {
            minor: true,
            search_status: DbSearch::Closed,
            ..row(&crypto)
        };
        let (loaded, ..) = build(minor, &crypto, NOW).unwrap();
        assert!(loaded.card.minor);
        assert!(loaded.published && !loaded.searchable);
        assert_eq!(loaded.card.search_status, SearchStatus::Closed);
    }

    #[test]
    fn cached_card_serves_only_while_the_privacy_flags_are_unchanged() {
        let crypto = crypto();
        let r = row(&crypto);
        let current = live(&r);
        let (loaded, shown, specialization) = build(r, &crypto, NOW).unwrap();
        assert_eq!(specialization, Some(Specialization::Backend));
        let cached = CachedCard {
            card: loaded.card,
            shown,
        };
        assert!(cached.reusable(&current));
        for changed in [
            db::pool::CardLive {
                show_name: false,
                ..current.clone()
            },
            db::pool::CardLive {
                show_fsp: false,
                ..current.clone()
            },
            db::pool::CardLive {
                show_experience: false,
                ..current.clone()
            },
        ] {
            assert!(!cached.reusable(&changed));
        }
        assert!(
            cached.reusable(&db::pool::CardLive {
                search_status: DbSearch::Closed,
                published: false,
                ..current
            }),
            "visibility is read live on every display, not part of the cached card"
        );
    }

    #[test]
    fn cached_card_takes_its_live_columns_from_the_live_row() {
        let crypto = crypto();
        let r = row(&crypto);
        let (loaded, shown, _) = build(r.clone(), &crypto, NOW).unwrap();
        let cached = CachedCard {
            card: loaded.card,
            shown,
        };
        let shown = cached.show(db::pool::CardLive {
            search_status: DbSearch::Closed,
            last_activity_at: at("2026-10-07T09:00:00Z"),
            minor: true,
            published: false,
            invitation_status: Some(Db::Accepted),
            applied: false,
            ..live(&r)
        });
        assert_eq!(shown.card.search_status, SearchStatus::Closed);
        assert_eq!(shown.card.last_activity_at, at("2026-10-07T09:00:00Z"));
        assert!(shown.card.minor);
        assert_eq!(
            shown.card.relation.invitation_status,
            Some(InvitationStatus::Accepted)
        );
        assert!(shown.card.relation.contacts_available);
        assert!(!shown.published && !shown.searchable);
        assert_eq!(shown.card.code, "C-0001");
        assert_eq!(shown.card.display_name.as_deref(), Some("Анна Смирнова"));

        let open = cached.show(db::pool::CardLive {
            search_status: DbSearch::Open,
            ..live(&r)
        });
        assert!(open.published && open.searchable);
        assert!(!open.card.minor && !open.card.relation.contacts_available);
    }

    #[test]
    fn hidden_profile_stays_open_to_a_company_in_touch() {
        let crypto = crypto();
        let card = |published, invitation, applied| {
            let (loaded, ..) = build(
                db::pool::CandidateCard {
                    published,
                    invitation_status: invitation,
                    applied,
                    ..row(&crypto)
                },
                &crypto,
                NOW,
            )
            .unwrap();
            loaded
        };
        assert!(card(true, None, false).visible_to_company());
        assert!(!card(false, None, false).visible_to_company());
        assert!(card(false, None, true).visible_to_company());
        assert!(card(false, Some(Db::Declined), false).visible_to_company());
    }
}
