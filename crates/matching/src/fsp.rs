//! FSP (Federation of Sport Programming) achievements and the FSP score (architecture §7).

use catalog::Specialization;
use serde::{Deserialize, Deserializer, Serialize};

use crate::ru;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum AchievementKind {
    Rank,
    Competition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
    pub const ALL: &'static [Discipline] = &[
        Discipline::Algorithmic,
        Discipline::Product,
        Discipline::Security,
        Discipline::Ai,
        Discipline::Robotics,
        Discipline::Uav,
    ];

    pub const fn slug(self) -> &'static str {
        match self {
            Discipline::Algorithmic => "algorithmic",
            Discipline::Product => "product",
            Discipline::Security => "security",
            Discipline::Ai => "ai",
            Discipline::Robotics => "robotics",
            Discipline::Uav => "uav",
        }
    }

    pub const fn name_ru(self) -> &'static str {
        match self {
            Discipline::Algorithmic => "Алгоритмическое программирование",
            Discipline::Product => "Продуктовое программирование",
            Discipline::Security => "Программирование систем информационной безопасности",
            Discipline::Ai => "Программирование систем ИИ",
            Discipline::Robotics => "Программирование робототехники",
            Discipline::Uav => "Программирование БАС",
        }
    }

    /// Form used after «по»: «соревнования по продуктовому программированию».
    pub const fn dative_ru(self) -> &'static str {
        match self {
            Discipline::Algorithmic => "алгоритмическому программированию",
            Discipline::Product => "продуктовому программированию",
            Discipline::Security => "программированию систем информационной безопасности",
            Discipline::Ai => "программированию систем ИИ",
            Discipline::Robotics => "программированию робототехники",
            Discipline::Uav => "программированию БАС",
        }
    }

    /// How much a result in this discipline says about a specialization. Product
    /// programming is team software delivery, so it maps fully onto backend/frontend;
    /// algorithmic contests show problem solving that matters most for backend and ML;
    /// security CTFs map onto security and partly devops/backend. Never 0: any FSP result
    /// still shows discipline and verified effort.
    pub const fn relevance(self, spec: Specialization) -> f32 {
        use Specialization as S;
        match (self, spec) {
            (Discipline::Algorithmic, S::Backend | S::Ml) => 0.8,
            (Discipline::Algorithmic, S::Frontend | S::Security) => 0.6,
            (Discipline::Algorithmic, S::Qa) => 0.5,
            (Discipline::Algorithmic, S::Devops) => 0.4,
            (Discipline::Product, S::Backend | S::Frontend) => 1.0,
            (Discipline::Product, S::Devops | S::Qa) => 0.7,
            (Discipline::Product, S::Ml) => 0.6,
            (Discipline::Product, S::Security) => 0.4,
            (Discipline::Security, S::Security) => 1.0,
            (Discipline::Security, S::Devops) => 0.6,
            (Discipline::Security, S::Backend) => 0.5,
            (Discipline::Security, S::Qa) => 0.4,
            (Discipline::Security, S::Frontend) => 0.3,
            (Discipline::Security, S::Ml) => 0.2,
            (Discipline::Ai, S::Ml) => 1.0,
            (Discipline::Ai, S::Backend) => 0.4,
            (Discipline::Ai, S::Devops | S::Qa | S::Security) => 0.3,
            (Discipline::Ai, S::Frontend) => 0.2,
            (Discipline::Robotics, S::Ml) => 0.6,
            (Discipline::Robotics, S::Backend | S::Devops | S::Qa) => 0.4,
            (Discipline::Robotics, S::Security) => 0.3,
            (Discipline::Robotics, S::Frontend) => 0.2,
            (Discipline::Uav, S::Ml) => 0.5,
            (Discipline::Uav, S::Qa | S::Security) => 0.4,
            (Discipline::Uav, S::Backend | S::Devops) => 0.3,
            (Discipline::Uav, S::Frontend) => 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Youth,
    #[serde(alias = "iii", alias = "3")]
    Third,
    #[serde(alias = "ii", alias = "2")]
    Second,
    #[serde(alias = "i", alias = "1")]
    First,
    #[serde(alias = "kms")]
    Cms,
    #[serde(alias = "msmk")]
    Ms,
}

impl Rank {
    /// Weights from architecture §7.3; a rank stands in for level × place.
    pub const fn weight(self) -> f32 {
        match self {
            Rank::Ms => 1.0,
            Rank::Cms => 0.8,
            Rank::First => 0.6,
            Rank::Second => 0.4,
            Rank::Third => 0.25,
            Rank::Youth => 0.1,
        }
    }

    pub const fn title_ru(self) -> &'static str {
        match self {
            Rank::Ms => "Мастер спорта (МС)",
            Rank::Cms => "Кандидат в мастера спорта (КМС)",
            Rank::First => "I спортивный разряд",
            Rank::Second => "II спортивный разряд",
            Rank::Third => "III спортивный разряд",
            Rank::Youth => "Юношеский разряд",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Regional,
    Interregional,
    #[serde(alias = "federal", alias = "all_russian")]
    National,
    International,
}

impl EventLevel {
    pub const fn weight(self) -> f32 {
        match self {
            EventLevel::Regional => 0.3,
            EventLevel::Interregional => 0.5,
            EventLevel::National => 0.8,
            EventLevel::International => 1.0,
        }
    }

    /// «… всероссийских соревнований»
    pub const fn genitive_ru(self) -> &'static str {
        match self {
            EventLevel::Regional => "региональных",
            EventLevel::Interregional => "межрегиональных",
            EventLevel::National => "всероссийских",
            EventLevel::International => "международных",
        }
    }
}

/// Serialized as slugs; also accepts places as numbers (`1`, `"2"`) from the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Place {
    First,
    Second,
    Third,
    Finalist,
    Participant,
}

impl Place {
    pub const fn weight(self) -> f32 {
        match self {
            Place::First => 1.0,
            Place::Second => 0.8,
            Place::Third => 0.7,
            Place::Finalist => 0.4,
            Place::Participant => 0.15,
        }
    }

    pub const fn title_ru(self) -> &'static str {
        match self {
            Place::First => "Победитель (1 место)",
            Place::Second => "Призёр (2 место)",
            Place::Third => "Призёр (3 место)",
            Place::Finalist => "Финалист",
            Place::Participant => "Участник",
        }
    }

    fn from_text(s: &str) -> Option<Place> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "first" | "1" | "winner" => Place::First,
            "second" | "2" => Place::Second,
            "third" | "3" => Place::Third,
            "finalist" => Place::Finalist,
            "participant" => Place::Participant,
            _ => return None,
        })
    }
}

impl<'de> Deserialize<'de> for Place {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Num(u64),
            Text(String),
        }
        let text = match Repr::deserialize(d)? {
            Repr::Num(n) => n.to_string(),
            Repr::Text(s) => s,
        };
        Place::from_text(&text)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown place `{text}`")))
    }
}

/// One record of the FSP registry snapshot (architecture §7.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Achievement {
    pub kind: AchievementKind,
    pub discipline: Discipline,
    #[serde(default)]
    pub rank: Option<Rank>,
    #[serde(default)]
    pub event_name: Option<String>,
    #[serde(default)]
    pub event_level: Option<EventLevel>,
    #[serde(default)]
    pub place: Option<Place>,
    #[serde(default)]
    pub team: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    /// `YYYY-MM-DD`
    pub date: String,
    #[serde(default)]
    pub protocol_url: Option<String>,
}

impl Achievement {
    /// Level × place for competitions, the rank weight for ranks. Missing level/place are
    /// read as the least prestigious option rather than dropped.
    pub fn base_weight(&self) -> f32 {
        match self.kind {
            AchievementKind::Rank => self.rank.map_or(0.0, Rank::weight),
            AchievementKind::Competition => {
                self.event_level.unwrap_or(EventLevel::Regional).weight()
                    * self.place.unwrap_or(Place::Participant).weight()
            }
        }
    }

    pub fn year(&self) -> Option<i32> {
        parse_date(&self.date).map(|d| d.year())
    }

    /// «Призёр (2 место) всероссийских соревнований по продуктовому программированию, 2025».
    pub fn describe_ru(&self) -> String {
        let discipline = self.discipline.dative_ru();
        let mut s = match self.kind {
            AchievementKind::Rank => {
                let title = self.rank.map_or("Спортивный разряд", Rank::title_ru);
                format!("{title} по {discipline}")
            }
            AchievementKind::Competition => {
                let who = self.place.map_or("Участник", Place::title_ru);
                match self.event_level {
                    Some(level) => {
                        format!("{who} {} соревнований по {discipline}", level.genitive_ru())
                    }
                    None => format!("{who} соревнований по {discipline}"),
                }
            }
        };
        if let Some(name) = self
            .event_name
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
        {
            s.push_str(&format!(" («{name}»)"));
        }
        if let Some(year) = self.year() {
            s.push_str(&format!(", {year}"));
        }
        s
    }
}

fn parse_date(s: &str) -> Option<time::Date> {
    let mut parts = s.trim().splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next().map_or(Some(7), |m| m.parse().ok())?;
    let day: u8 = parts
        .next()
        .map_or(Some(1), |d| d.get(..2).unwrap_or(d).parse().ok())?;
    time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()
}

const UNIX_EPOCH_JULIAN_DAY: i64 = 2_440_588;
const HALF_LIFE_YEARS: f32 = 2.0;
/// Unparseable dates are treated as two years old: still counted, never as fresh.
const UNKNOWN_DATE_DECAY: f32 = 0.5;

fn decay(date: &str, now_unix: i64) -> f32 {
    let Some(d) = parse_date(date) else {
        return UNKNOWN_DATE_DECAY;
    };
    let days = now_unix.div_euclid(86_400) + UNIX_EPOCH_JULIAN_DAY - i64::from(d.to_julian_day());
    let years = days.max(0) as f32 / 365.25;
    0.5f32.powf(years / HALF_LIFE_YEARS)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Contribution {
    /// Index into the achievements slice passed to [`fsp_score`].
    pub index: u32,
    pub value: f32,
    pub explanation_ru: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FspScore {
    /// 0..1
    pub score: f32,
    pub has_history: bool,
    /// Sorted by value, highest first.
    pub contributions: Vec<Contribution>,
    pub summary_ru: String,
}

pub const NO_FSP_DATA_RU: &str = "Нет данных ФСП";

impl FspScore {
    /// Candidate without FSP history (F-3): zero score, neutral wording, no penalty.
    pub fn none() -> Self {
        FspScore {
            score: 0.0,
            has_history: false,
            contributions: Vec::new(),
            summary_ru: NO_FSP_DATA_RU.to_owned(),
        }
    }
}

impl Default for FspScore {
    fn default() -> Self {
        Self::none()
    }
}

/// `fsp = 1 − Π(1 − vᵢ)`, `v = base × relevance(discipline, spec) × 0.5^(age / 2 years)`.
/// Noisy-OR grows with every result yet saturates below 1, so many small results never
/// outweigh the FSP feature's fixed share of the matching score. Ranks are cumulative, so
/// only the highest rank per discipline counts.
pub fn fsp_score(achievements: &[Achievement], spec: Specialization, now_unix: i64) -> FspScore {
    if achievements.is_empty() {
        return FspScore::none();
    }
    let mut best_rank: [Option<usize>; 6] = [None; 6];
    for (i, a) in achievements.iter().enumerate() {
        if a.kind != AchievementKind::Rank {
            continue;
        }
        let slot = &mut best_rank[a.discipline as usize];
        let better = slot.is_none_or(|j| {
            let b = &achievements[j];
            (a.base_weight(), &a.date) > (b.base_weight(), &b.date)
        });
        if better {
            *slot = Some(i);
        }
    }
    let mut keep = 1.0f32;
    let mut contributions: Vec<Contribution> = achievements
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let superseded =
                a.kind == AchievementKind::Rank && best_rank[a.discipline as usize] != Some(i);
            let relevance = a.discipline.relevance(spec);
            let value = if superseded {
                0.0
            } else {
                (a.base_weight() * relevance * decay(&a.date, now_unix)).clamp(0.0, 1.0)
            };
            keep *= 1.0 - value;
            let mut explanation_ru = a.describe_ru();
            if superseded {
                explanation_ru.push_str(" — учтён более высокий разряд");
            } else if relevance <= 0.4 {
                explanation_ru.push_str(" — смежная дисциплина");
            }
            Contribution {
                index: i as u32,
                value,
                explanation_ru,
            }
        })
        .collect();
    contributions.sort_by(|a, b| b.value.total_cmp(&a.value).then(a.index.cmp(&b.index)));
    let summary_ru = match contributions.len() {
        0 => NO_FSP_DATA_RU.to_owned(),
        1 => contributions[0].explanation_ru.clone(),
        n => format!(
            "{}; ещё {} {}",
            contributions[0].explanation_ru,
            n - 1,
            ru::plural(n - 1, ["достижение", "достижения", "достижений"])
        ),
    };
    FspScore {
        score: 1.0 - keep,
        has_history: true,
        contributions,
        summary_ru,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_791_158_400; // 2026-10-05

    fn competition(d: Discipline, level: EventLevel, place: Place, date: &str) -> Achievement {
        Achievement {
            kind: AchievementKind::Competition,
            discipline: d,
            rank: None,
            event_name: None,
            event_level: Some(level),
            place: Some(place),
            team: None,
            role: None,
            date: date.into(),
            protocol_url: None,
        }
    }

    fn rank(d: Discipline, r: Rank, date: &str) -> Achievement {
        Achievement {
            kind: AchievementKind::Rank,
            rank: Some(r),
            event_level: None,
            place: None,
            ..competition(d, EventLevel::Regional, Place::Participant, date)
        }
    }

    #[test]
    fn no_history_is_neutral() {
        let s = fsp_score(&[], Specialization::Backend, NOW);
        assert_eq!(s.score, 0.0);
        assert!(!s.has_history);
        assert_eq!(s.summary_ru, "Нет данных ФСП");
        assert_eq!(FspScore::default(), s);
    }

    #[test]
    fn explanation_text() {
        let a = competition(
            Discipline::Product,
            EventLevel::National,
            Place::Second,
            "2025-11-20",
        );
        assert_eq!(
            a.describe_ru(),
            "Призёр (2 место) всероссийских соревнований по продуктовому программированию, 2025"
        );
        let r = rank(Discipline::Algorithmic, Rank::Cms, "2024-03-01");
        assert_eq!(
            r.describe_ru(),
            "Кандидат в мастера спорта (КМС) по алгоритмическому программированию, 2024"
        );
    }

    #[test]
    fn decay_halves_every_two_years() {
        let fresh = competition(
            Discipline::Product,
            EventLevel::International,
            Place::First,
            "2026-10-05",
        );
        let old = competition(
            Discipline::Product,
            EventLevel::International,
            Place::First,
            "2024-10-05",
        );
        let f = fsp_score(&[fresh], Specialization::Backend, NOW).score;
        let o = fsp_score(&[old], Specialization::Backend, NOW).score;
        assert!((f - 1.0).abs() < 1e-4, "{f}");
        assert!((o - 0.5).abs() < 0.01, "{o}");
    }

    #[test]
    fn relevance_depends_on_specialization() {
        let ctf = competition(
            Discipline::Security,
            EventLevel::National,
            Place::First,
            "2026-06-01",
        );
        let sec = fsp_score(std::slice::from_ref(&ctf), Specialization::Security, NOW);
        let front = fsp_score(std::slice::from_ref(&ctf), Specialization::Frontend, NOW);
        assert!(sec.score > 2.0 * front.score);
        assert!(front.summary_ru.contains("смежная дисциплина"));
    }

    #[test]
    fn noisy_or_saturates() {
        let one = competition(
            Discipline::Product,
            EventLevel::Regional,
            Place::Participant,
            "2026-09-01",
        );
        let many = vec![one.clone(); 40];
        let s1 = fsp_score(std::slice::from_ref(&one), Specialization::Backend, NOW).score;
        let s40 = fsp_score(&many, Specialization::Backend, NOW).score;
        assert!(s40 > s1);
        assert!(s40 < 1.0);
        let two = fsp_score(&[one.clone(), one], Specialization::Backend, NOW);
        assert!((two.score - (1.0 - (1.0 - s1).powi(2))).abs() < 1e-5);
        assert!(two.summary_ru.contains("ещё 1 достижение"));
    }

    #[test]
    fn only_highest_rank_counts() {
        let list = [
            rank(Discipline::Algorithmic, Rank::Second, "2022-01-01"),
            rank(Discipline::Algorithmic, Rank::Cms, "2025-01-01"),
            rank(Discipline::Algorithmic, Rank::First, "2023-01-01"),
        ];
        let s = fsp_score(&list, Specialization::Backend, NOW);
        let solo = fsp_score(&list[1..2], Specialization::Backend, NOW);
        assert!((s.score - solo.score).abs() < 1e-6);
        assert_eq!(s.contributions[0].index, 1);
        assert!(
            s.contributions[2]
                .explanation_ru
                .contains("учтён более высокий разряд")
        );
    }

    #[test]
    fn deserializes_registry_json() {
        let json = r#"[
            {"kind":"competition","discipline":"product","event_name":"Кубок России","event_level":"national",
             "place":2,"team":"Null Pointers","role":"капитан","date":"2025-11-20","protocol_url":"https://fsp-russia.com/p/1","id":"x"},
            {"kind":"rank","discipline":"algorithmic","rank":"cms","date":"2024-03-01"},
            {"kind":"competition","discipline":"ai","event_level":"regional","place":"finalist","date":"2025"}
        ]"#;
        let list: Vec<Achievement> = serde_json::from_str(json).unwrap();
        assert_eq!(list[0].place, Some(Place::Second));
        assert_eq!(list[1].rank, Some(Rank::Cms));
        assert_eq!(list[2].year(), Some(2025));
        let s = fsp_score(&list, Specialization::Backend, NOW);
        assert!(s.score > 0.0 && s.score < 1.0);
        assert!(s.summary_ru.starts_with("Призёр (2 место) всероссийских соревнований по продуктовому программированию («Кубок России»), 2025"));
        assert_eq!(serde_json::to_value(Place::First).unwrap(), "first");
    }
}
