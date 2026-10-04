//! Registry data model. The achievement vocabulary is shared with the `matching` crate
//! (architecture.md §7.2): enum spellings here are part of the contract.

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementKind {
    Rank,
    Competition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Discipline {
    Algorithmic,
    Product,
    Security,
    Ai,
    Robotics,
    Uav,
}

impl Discipline {
    pub const COUNT: usize = 6;
}

/// Declared from lowest to highest so that `Ord` follows seniority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Youth,
    Third,
    Second,
    First,
    Cms,
    Ms,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Regional,
    Interregional,
    National,
    International,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Place {
    First,
    Second,
    Third,
    Finalist,
    Participant,
}

/// Fields that do not apply to the record kind are serialized as `null`, never omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    pub id: String,
    pub kind: AchievementKind,
    pub discipline: Discipline,
    pub rank: Option<Rank>,
    pub event_name: Option<String>,
    pub event_level: Option<EventLevel>,
    pub place: Option<Place>,
    pub team: Option<String>,
    pub role: Option<String>,
    pub date: String,
    pub protocol_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub fsp_id: String,
    pub last_name: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub region: String,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RankSummary {
    pub discipline: Discipline,
    pub rank: Rank,
    pub assigned_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParticipantCard<'a> {
    pub fsp_id: &'a str,
    pub full_name: String,
    pub last_name: &'a str,
    pub first_name: &'a str,
    pub middle_name: Option<&'a str>,
    pub region: &'a str,
    pub ranks: Vec<RankSummary>,
    pub competitions_count: usize,
    pub last_activity: Option<&'a str>,
}

#[derive(Serialize)]
struct AchievementsPayload<'a> {
    fsp_id: &'a str,
    achievements: &'a [Achievement],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AchievementError {
    #[error("empty id")]
    EmptyId,
    #[error("date must be YYYY-MM-DD")]
    InvalidDate,
    #[error("protocol_url must be an https URL")]
    InsecureProtocolUrl,
    #[error("rank record without rank")]
    MissingRank,
    #[error("rank record must not carry event fields")]
    RankWithEventFields,
    #[error("competition record must not carry a rank")]
    CompetitionWithRank,
    #[error("competition record needs event_name, event_level and place")]
    IncompleteCompetition,
    #[error("role without team")]
    RoleWithoutTeam,
}

/// Raised only while loading the embedded data at startup, never per request.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataError {
    #[error("participants JSON is malformed: {0}")]
    Json(String),
    #[error("invalid fsp_id {0:?}, expected FSP-NNNNNN")]
    InvalidFspId(String),
    #[error("{fsp_id}: {field} is empty")]
    EmptyField { fsp_id: String, field: &'static str },
    #[error("{fsp_id} / {achievement_id}: {reason}")]
    InvalidAchievement {
        fsp_id: String,
        achievement_id: String,
        reason: AchievementError,
    },
    #[error("duplicate fsp_id {0}")]
    DuplicateFspId(String),
    #[error("duplicate achievement id {0}")]
    DuplicateAchievementId(String),
}

impl Participant {
    pub fn full_name(&self) -> String {
        match self.middle_name.as_deref() {
            Some(middle) => format!("{} {} {}", self.last_name, self.first_name, middle),
            None => format!("{} {}", self.last_name, self.first_name),
        }
    }

    pub fn card(&self) -> ParticipantCard<'_> {
        let mut best: [Option<(Rank, &str, Discipline)>; Discipline::COUNT] = [None; Discipline::COUNT];
        for a in &self.achievements {
            let Some(rank) = a.rank.filter(|_| a.kind == AchievementKind::Rank) else {
                continue;
            };
            let slot = &mut best[a.discipline as usize];
            if slot.is_none_or(|(r, d, _)| (rank, a.date.as_str()) > (r, d)) {
                *slot = Some((rank, a.date.as_str(), a.discipline));
            }
        }
        let mut ranks: Vec<RankSummary> = best
            .into_iter()
            .flatten()
            .map(|(rank, date, discipline)| RankSummary {
                discipline,
                rank,
                assigned_at: date.to_owned(),
            })
            .collect();
        ranks.sort_by(|a, b| b.rank.cmp(&a.rank).then(a.discipline.cmp(&b.discipline)));

        ParticipantCard {
            fsp_id: &self.fsp_id,
            full_name: self.full_name(),
            last_name: &self.last_name,
            first_name: &self.first_name,
            middle_name: self.middle_name.as_deref(),
            region: &self.region,
            ranks,
            competitions_count: self
                .achievements
                .iter()
                .filter(|a| a.kind == AchievementKind::Competition)
                .count(),
            last_activity: self.achievements.iter().map(|a| a.date.as_str()).max(),
        }
    }

    fn validate(&self) -> Result<(), DataError> {
        if !is_fsp_id(&self.fsp_id) {
            return Err(DataError::InvalidFspId(self.fsp_id.clone()));
        }
        for (field, value) in [
            ("last_name", &self.last_name),
            ("first_name", &self.first_name),
            ("region", &self.region),
        ] {
            if value.trim().is_empty() {
                return Err(DataError::EmptyField {
                    fsp_id: self.fsp_id.clone(),
                    field,
                });
            }
        }
        for a in &self.achievements {
            a.validate()
                .map_err(|reason| DataError::InvalidAchievement {
                    fsp_id: self.fsp_id.clone(),
                    achievement_id: a.id.clone(),
                    reason,
                })?;
        }
        Ok(())
    }
}

impl Achievement {
    fn validate(&self) -> Result<(), AchievementError> {
        if self.id.trim().is_empty() {
            return Err(AchievementError::EmptyId);
        }
        if !is_iso_date(&self.date) {
            return Err(AchievementError::InvalidDate);
        }
        if self
            .protocol_url
            .as_deref()
            .is_some_and(|url| !url.starts_with("https://"))
        {
            return Err(AchievementError::InsecureProtocolUrl);
        }
        match self.kind {
            AchievementKind::Rank => {
                if self.rank.is_none() {
                    return Err(AchievementError::MissingRank);
                }
                if self.event_name.is_some()
                    || self.event_level.is_some()
                    || self.place.is_some()
                    || self.team.is_some()
                    || self.role.is_some()
                {
                    return Err(AchievementError::RankWithEventFields);
                }
            }
            AchievementKind::Competition => {
                if self.rank.is_some() {
                    return Err(AchievementError::CompetitionWithRank);
                }
                if self
                    .event_name
                    .as_deref()
                    .is_none_or(|n| n.trim().is_empty())
                    || self.event_level.is_none()
                    || self.place.is_none()
                {
                    return Err(AchievementError::IncompleteCompetition);
                }
                if self.role.is_some() && self.team.is_none() {
                    return Err(AchievementError::RoleWithoutTeam);
                }
            }
        }
        Ok(())
    }
}

/// A participant together with its response bodies, rendered once at load time.
#[derive(Debug)]
pub struct Entry {
    participant: Participant,
    card_json: Box<str>,
    achievements_json: Box<str>,
}

impl Entry {
    pub fn participant(&self) -> &Participant {
        &self.participant
    }

    pub fn card_json(&self) -> &str {
        &self.card_json
    }

    pub fn achievements_json(&self) -> &str {
        &self.achievements_json
    }
}

/// Immutable after construction. Lookups touch only the small `index`; the bulky entries are
/// read once a participant is found.
#[derive(Debug)]
pub struct Registry {
    index: FxHashMap<Box<str>, usize>,
    entries: Box<[Entry]>,
}

static EMBEDDED: LazyLock<Result<Registry, DataError>> =
    LazyLock::new(|| Registry::from_json(include_str!("../data/participants.json")));

impl Registry {
    /// The registry compiled from `data/participants.json`, parsed on first use.
    pub fn embedded() -> Result<&'static Registry, DataError> {
        EMBEDDED.as_ref().map_err(Clone::clone)
    }

    pub fn from_json(json: &str) -> Result<Self, DataError> {
        let list: Vec<Participant> =
            serde_json::from_str(json).map_err(|e| DataError::Json(e.to_string()))?;
        let mut index = FxHashMap::with_capacity_and_hasher(list.len(), Default::default());
        let mut entries = Vec::with_capacity(list.len());
        let mut achievement_ids = FxHashSet::default();
        for mut participant in list {
            participant.validate()?;
            for a in &participant.achievements {
                if !achievement_ids.insert(a.id.clone()) {
                    return Err(DataError::DuplicateAchievementId(a.id.clone()));
                }
            }
            participant
                .achievements
                .sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
            if index
                .insert(participant.fsp_id.clone().into_boxed_str(), entries.len())
                .is_some()
            {
                return Err(DataError::DuplicateFspId(participant.fsp_id));
            }
            entries.push(Entry {
                card_json: render(&participant.card())?,
                achievements_json: render(&AchievementsPayload {
                    fsp_id: &participant.fsp_id,
                    achievements: &participant.achievements,
                })?,
                participant,
            });
        }
        Ok(Self {
            index,
            entries: entries.into_boxed_slice(),
        })
    }

    /// Matches the ID case-insensitively, so the lowercase Keycloak username works too.
    pub fn get(&self, fsp_id: &str) -> Option<&Entry> {
        let id = fsp_id.trim();
        let position = if id.bytes().any(|b| b.is_ascii_lowercase()) {
            self.index.get(id.to_ascii_uppercase().as_str())
        } else {
            self.index.get(id)
        };
        position.and_then(|&i| self.entries.get(i))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn participants(&self) -> impl Iterator<Item = &Participant> {
        self.entries.iter().map(Entry::participant)
    }
}

fn render(value: &impl Serialize) -> Result<Box<str>, DataError> {
    serde_json::to_string(value)
        .map(String::into_boxed_str)
        .map_err(|e| DataError::Json(e.to_string()))
}

fn is_fsp_id(s: &str) -> bool {
    s.strip_prefix("FSP-")
        .is_some_and(|digits| digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |range: std::ops::Range<usize>| -> Option<u32> {
        let part = &s[range];
        if part.bytes().all(|c| c.is_ascii_digit()) {
            part.parse().ok()
        } else {
            None
        }
    };
    let (Some(year), Some(month), Some(day)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days_in_month).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> &'static Registry {
        Registry::embedded().expect("embedded registry data must be valid")
    }

    #[test]
    fn embedded_data_matches_demo_expectations() {
        let r = registry();
        assert_eq!(r.len(), 25);

        let empty = r
            .participants()
            .filter(|p| p.achievements.is_empty())
            .count();
        assert_eq!(empty, 3);

        let single = r
            .participants()
            .filter(|p| p.achievements.len() == 1)
            .count();
        assert!(single >= 3);

        let top_rank = |p: &Participant| p.card().ranks.first().map(|s| s.rank);
        assert!(r.participants().any(|p| top_rank(p) == Some(Rank::Ms)));
        assert!(r.participants().any(|p| top_rank(p) == Some(Rank::Cms)));

        let mut seen = [false; Discipline::COUNT];
        for a in r.participants().flat_map(|p| p.achievements.iter()) {
            seen[a.discipline as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));

        for p in r.participants() {
            for a in &p.achievements {
                assert!(
                    ("2021-01-01".."2026-10-01").contains(&a.date.as_str()),
                    "{} {} is outside 2021..today",
                    p.fsp_id,
                    a.date
                );
            }
        }
    }

    #[test]
    fn card_summarizes_highest_rank_per_discipline() {
        let r = registry();
        let entry = r.get("fsp-000001").expect("lookup is case-insensitive");
        let card = entry.participant().card();
        assert_eq!(card.full_name, "Смирнов Алексей Игоревич");
        assert_eq!(
            card.ranks,
            vec![RankSummary {
                discipline: Discipline::Algorithmic,
                rank: Rank::Ms,
                assigned_at: "2024-12-16".into(),
            }]
        );
        assert_eq!(card.competitions_count, 6);
        assert_eq!(card.last_activity, Some("2025-11-23"));

        let empty = r.get("FSP-000010").unwrap().participant().card();
        assert!(empty.ranks.is_empty());
        assert_eq!(empty.competitions_count, 0);
        assert_eq!(empty.last_activity, None);
    }

    #[test]
    fn prerendered_bodies_match_the_model() {
        let entry = registry().get("FSP-000002").unwrap();
        let card: serde_json::Value = serde_json::from_str(entry.card_json()).unwrap();
        assert_eq!(card["full_name"], "Кузнецова Мария Андреевна");
        assert_eq!(card["ranks"][0]["rank"], "cms");

        let body: serde_json::Value = serde_json::from_str(entry.achievements_json()).unwrap();
        assert_eq!(body["fsp_id"], "FSP-000002");
        let list: Vec<Achievement> = serde_json::from_value(body["achievements"].clone()).unwrap();
        assert_eq!(list, entry.participant().achievements);
        assert!(list.windows(2).all(|w| w[0].date >= w[1].date));
        assert!(
            body["achievements"][0]
                .get("rank")
                .is_some_and(|v| v.is_null())
        );
    }

    #[test]
    fn rejects_inconsistent_records() {
        let base = r#"[{"fsp_id":"FSP-000001","last_name":"Тест","first_name":"Тест","middle_name":null,
            "region":"Москва","achievements":[{"id":"x","kind":"rank","discipline":"ai","rank":"first",
            "event_name":null,"event_level":null,"place":PLACE,"team":null,"role":null,
            "date":"DATE","protocol_url":null}]}]"#;
        let ok = base.replace("PLACE", "null").replace("DATE", "2024-02-29");
        assert!(Registry::from_json(&ok).is_ok());

        let with_place = base
            .replace("PLACE", "\"first\"")
            .replace("DATE", "2024-02-29");
        assert!(matches!(
            Registry::from_json(&with_place),
            Err(DataError::InvalidAchievement {
                reason: AchievementError::RankWithEventFields,
                ..
            })
        ));

        let bad_date = base.replace("PLACE", "null").replace("DATE", "2023-02-29");
        assert!(matches!(
            Registry::from_json(&bad_date),
            Err(DataError::InvalidAchievement {
                reason: AchievementError::InvalidDate,
                ..
            })
        ));

        let unknown_rank = ok.replace("\"first\"", "\"grandmaster\"");
        assert!(matches!(
            Registry::from_json(&unknown_rank),
            Err(DataError::Json(_))
        ));

        let one = ok.trim().trim_start_matches('[').trim_end_matches(']');
        assert!(matches!(
            Registry::from_json(&format!("[{one},{one}]")),
            Err(DataError::DuplicateAchievementId(_))
        ));
    }

    #[test]
    fn iso_date_validation() {
        assert!(is_iso_date("2026-01-31"));
        assert!(is_iso_date("2024-02-29"));
        assert!(!is_iso_date("2025-02-29"));
        assert!(!is_iso_date("2025-13-01"));
        assert!(!is_iso_date("2025-1-01"));
        assert!(!is_iso_date("2025-01-00"));
        assert!(!is_iso_date("+025-01-01"));
    }
}
