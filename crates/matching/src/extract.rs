//! Numeric and classification extractors shared by the need and resume parsers.
//! All regexes run on folded text (lowercase, `ё`→`е`, dashes → `-`).

use std::sync::LazyLock;

use catalog::{Grade, Specialization};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::lexicon;
use crate::skills::{self, SkillId};

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex is valid")
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpecializationGuess {
    pub specialization: Specialization,
    /// 0..1: share of the evidence that points at this specialization, damped when the
    /// evidence itself is thin.
    pub confidence: f32,
}

/// Role words weigh 3 (doubled in the title), each distinct skill adds its affinity vote.
/// Confidence = top share × min(1, top / 4): a lone "React" mention is not enough to be sure.
pub(crate) fn guess_specialization(
    title: &str,
    body: &str,
    skills: &[SkillId],
) -> Option<SpecializationGuess> {
    let mut score = [0.0f32; 6];
    for (text, factor) in [(title, 2.0f32), (body, 1.0)] {
        for (spec, w) in lexicon::SPEC_MATCHER.find_str(text).into_iter().flatten() {
            score[*spec as usize] += w * factor;
        }
    }
    for &id in skills {
        for (s, v) in score.iter_mut().zip(skills::specialization_affinity(id)) {
            *s += v;
        }
    }
    let total: f32 = score.iter().sum();
    let (best, &top) = score.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1))?;
    if top <= 0.0 {
        return None;
    }
    let confidence = (top / total) * (top / 4.0).min(1.0);
    Some(SpecializationGuess {
        specialization: *Specialization::ALL.get(best)?,
        confidence: (confidence * 100.0).round() / 100.0,
    })
}

/// Typical market mapping of total experience to grade (architecture §4: junior ≈ up to 2 years).
pub(crate) fn grade_for_experience(years: f32) -> Grade {
    if years < 2.0 {
        Grade::Junior
    } else if years < 5.0 {
        Grade::Middle
    } else {
        Grade::Senior
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SalaryRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

static SALARY: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?x)
        (?:\b(?P<pre>от|from|до|up\ to|to)\s+)?
        (?P<a>\d{1,3}(?:[\ ,]\d{3})+|\d+(?:[.,]\d+)?)
        \s*(?P<am>тыс\.?|тысяч\w*|т\.?\s?р\b\.?|k\b|к\b|млн\.?|m\b)?
        \s*(?P<c1>₽|руб\w*\.?|р\.|rub\b|rur\b)?
        (?:\s*(?:-|до|to)\s*
            (?P<b>\d{1,3}(?:[\ ,]\d{3})+|\d+(?:[.,]\d+)?)
            \s*(?P<bm>тыс\.?|тысяч\w*|т\.?\s?р\b\.?|k\b|к\b|млн\.?|m\b)?
        )?
        \s*(?P<c2>₽|руб\w*\.?|р\.|rub\b|rur\b)?")
});

const SALARY_WORDS: &[&str] = &[
    "зарплат",
    "з/п",
    "зп",
    "оклад",
    "вилк",
    "доход",
    "компенсац",
    "salary",
    "ожидани",
    "на руки",
    "gross",
    "net",
];

const SALARY_FLOOR: f64 = 10_000.0;
const SALARY_CEIL: f64 = 10_000_000.0;

fn parse_amount(s: &str) -> Option<f64> {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let grouped = compact
        .rsplit_once([',', '.'])
        .is_some_and(|(_, tail)| tail.len() == 3 && compact.len() > 4);
    let normalized = if grouped {
        compact.replace([',', '.'], "")
    } else {
        compact.replace(',', ".")
    };
    normalized.parse().ok()
}

fn multiplier(m: Option<&str>) -> f64 {
    match m {
        None => 1.0,
        Some(s) if s.starts_with("млн") || s == "m" => 1_000_000.0,
        Some(_) => 1_000.0,
    }
}

/// First plausible monthly salary (range) in rubles. A number only counts as money when it
/// carries a multiplier, a currency sign or salary wording on the same line — otherwise
/// "от 3 лет" or "10 000 пользователей" would leak in.
pub(crate) fn salary(folded: &str) -> Option<SalaryRange> {
    for line in folded.lines() {
        for caps in SALARY.captures_iter(line) {
            if let Some(range) = salary_from(&caps, line) {
                return Some(range);
            }
        }
    }
    None
}

fn salary_from(caps: &Captures<'_>, line: &str) -> Option<SalaryRange> {
    let start = caps.get(0)?.start();
    let a = parse_amount(caps.name("a")?.as_str())?;
    let b = caps.name("b").and_then(|m| parse_amount(m.as_str()));
    let am = caps.name("am").map(|m| m.as_str());
    let bm = caps.name("bm").map(|m| m.as_str());
    let has_mult = am.is_some() || bm.is_some();
    let has_currency = caps.name("c1").is_some() || caps.name("c2").is_some();
    let ctx_before = SALARY_WORDS.iter().any(|w| line[..start].contains(w));
    let ctx_line = ctx_before || SALARY_WORDS.iter().any(|w| line.contains(w));
    if !(has_mult || has_currency || ctx_line) {
        return None;
    }
    // "200–300 тыс." — the multiplier written once applies to both ends.
    let am_value = if am.is_none() && bm.is_some() && a < 1_000.0 {
        multiplier(bm)
    } else {
        multiplier(am)
    };
    let mut av = a * am_value;
    let mut bv = b.map(|b| b * multiplier(bm));
    // "зарплата 200-300" — bare small numbers after salary wording mean thousands.
    if !has_mult && ctx_before {
        if av < 1_000.0 {
            av *= 1_000.0;
        }
        bv = bv.map(|v| if v < 1_000.0 { v * 1_000.0 } else { v });
    }
    let plausible = |v: f64| (SALARY_FLOOR..=SALARY_CEIL).contains(&v);
    if !plausible(av) || bv.is_some_and(|v| !plausible(v)) {
        return None;
    }
    let to_rub = |v: f64| v.round() as i64;
    Some(match (bv, caps.name("pre").map(|m| m.as_str())) {
        (Some(bv), _) => SalaryRange {
            min: Some(to_rub(av.min(bv))),
            max: Some(to_rub(av.max(bv))),
        },
        (None, Some("до" | "to" | "up to")) => SalaryRange {
            min: None,
            max: Some(to_rub(av)),
        },
        (None, Some(_)) => SalaryRange {
            min: Some(to_rub(av)),
            max: None,
        },
        (None, None) => SalaryRange {
            min: Some(to_rub(av)),
            max: Some(to_rub(av)),
        },
    })
}

const YEARS: &str = r"(?:год\w*|лет|years?|yrs?)";
const MONTHS: &str = r"(?:месяц\w*|мес\b\.?|months?)";

/// hh.ru-style total at the start of a line: "Опыт работы — 6 лет 4 месяца".
static EXPERIENCE_TOTAL: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?m)^\s*(?:общий\s+)?(?:опыт\s+работы|опыт|стаж(?:\s+работы)?|work\s+experience|total\s+experience|experience)\s*[:\-]?\s*(?:(?:более|больше|около|свыше|over|about|more\s+than)\s+)?(?:(?P<y>\d+(?:[.,]\d+)?)\+?\s*{YEARS})?(?:\s*(?:и\s+)?(?P<m>\d+)\s*{MONTHS})?"
    ))
});

/// "5 лет опыта", "3+ years of experience", "1,5 года коммерческого опыта".
static EXPERIENCE_BEFORE: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?P<y>\d+(?:[.,]\d+)?)\+?\s*{YEARS}\s+(?:\w+\s+){{0,2}}?(?:опыт\w*|experience|стаж\w*)"
    ))
});

/// "опыт коммерческой разработки на Go от 5 лет", "experience: 3-5 years".
static EXPERIENCE_AFTER: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?:опыт\w*|experience|стаж\w*)[^\d\n]{{0,40}}?(?P<y>\d+(?:[.,]\d+)?)\+?\s*(?:-\s*\d+\s*)?{YEARS}"
    ))
});

fn number(caps: &Captures<'_>, name: &str) -> Option<f32> {
    caps.name(name)?.as_str().replace(',', ".").parse().ok()
}

fn plausible_years(y: f32) -> Option<f32> {
    (0.0..=50.0)
        .contains(&y)
        .then_some((y * 10.0).round() / 10.0)
}

/// Explicit total-experience statement at a line start.
pub(crate) fn experience_total(folded: &str) -> Option<f32> {
    EXPERIENCE_TOTAL.captures_iter(folded).find_map(|c| {
        let y = number(&c, "y");
        let m = number(&c, "m");
        if y.is_none() && m.is_none() {
            return None;
        }
        plausible_years(y.unwrap_or(0.0) + m.unwrap_or(0.0) / 12.0)
    })
}

/// Experience mentioned anywhere in prose; ranges ("3-5 лет") yield their lower bound.
pub(crate) fn experience_mention(folded: &str) -> Option<f32> {
    if folded.contains("без опыта") {
        return Some(0.0);
    }
    [&*EXPERIENCE_AFTER, &*EXPERIENCE_BEFORE]
        .iter()
        .flat_map(|r| r.captures_iter(folded))
        .find_map(|c| number(&c, "y").and_then(plausible_years))
}

static DATE_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?x)
        (?:(?P<sm>[a-zа-я]{3,9})\.?\s+|(?P<smn>\d{1,2})[./])?(?P<sy>(?:19|20)\d{2})\b
        \s*(?:-|по|to|until|till)\s*
        (?:
            (?P<now>(?:по\s+)?настоящее\s+время|н\.?\s?в\b\.?|сейчас|текущ\w*\s+момент|present|now|current|today)
            |
            (?:(?P<em>[a-zа-я]{3,9})\.?\s+|(?P<emn>\d{1,2})[./])?(?P<ey>(?:19|20)\d{2})\b
        )")
});

fn month_from_word(w: &str) -> Option<u32> {
    const RU: [&str; 12] = [
        "янв", "фев", "мар", "апр", "ма", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
    ];
    const EN: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    if w == "май" || w == "мая" {
        return Some(5);
    }
    if let Some(i) = RU
        .iter()
        .position(|p| p.chars().count() == 3 && w.starts_with(p))
    {
        return Some(i as u32 + 1);
    }
    EN.iter()
        .position(|full| w == *full || (w.len() >= 3 && full.starts_with(w)))
        .map(|i| i as u32 + 1)
}

fn month_of(caps: &Captures<'_>, word: &str, num: &str) -> Option<u32> {
    caps.name(word)
        .and_then(|m| month_from_word(m.as_str()))
        .or_else(|| caps.name(num).and_then(|m| m.as_str().parse().ok()))
        .filter(|m| (1..=12).contains(m))
}

/// `(year, month)` of a unix timestamp.
pub(crate) fn year_month(now_unix: i64) -> (i32, u32) {
    time::OffsetDateTime::from_unix_timestamp(now_unix)
        .map(|t| (t.year(), u32::from(t.month() as u8)))
        .unwrap_or((1970, 1))
}

/// Total months covered by employment date ranges, overlaps merged. Year-only bounds are
/// taken as mid-year so "2019 — 2021" counts two years, not one or three.
pub(crate) fn employment_months<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    now_unix: i64,
) -> Option<u32> {
    let (ny, nm) = year_month(now_unix);
    let now_index = ny * 12 + nm as i32 - 1;
    let mut spans: Vec<(i32, i32)> = Vec::new();
    for line in lines {
        for c in DATE_RANGE.captures_iter(line) {
            let Some(sy) = c.name("sy").and_then(|m| m.as_str().parse::<i32>().ok()) else {
                continue;
            };
            let start = match month_of(&c, "sm", "smn") {
                Some(m) => sy * 12 + m as i32 - 1,
                None => sy * 12 + 6,
            };
            let end = if c.name("now").is_some() {
                now_index + 1
            } else {
                let Some(ey) = c.name("ey").and_then(|m| m.as_str().parse::<i32>().ok()) else {
                    continue;
                };
                match month_of(&c, "em", "emn") {
                    Some(m) => ey * 12 + m as i32,
                    None => ey * 12 + 6,
                }
            };
            let end = end.min(now_index + 1);
            if end > start && sy >= 1970 {
                spans.push((start, end));
            }
        }
    }
    if spans.is_empty() {
        return None;
    }
    spans.sort_unstable();
    let mut total = 0;
    let (mut cur_start, mut cur_end) = spans[0];
    for &(s, e) in &spans[1..] {
        if s <= cur_end {
            cur_end = cur_end.max(e);
        } else {
            total += cur_end - cur_start;
            (cur_start, cur_end) = (s, e);
        }
    }
    total += cur_end - cur_start;
    u32::try_from(total).ok().map(|t| t.min(600))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::fold;

    fn sal(s: &str) -> Option<(Option<i64>, Option<i64>)> {
        salary(&fold(s)).map(|r| (r.min, r.max))
    }

    #[test]
    fn salary_formats() {
        assert_eq!(
            sal("зарплата 200–300 тыс. ₽"),
            Some((Some(200_000), Some(300_000)))
        );
        assert_eq!(
            sal("от 250 000 до 350 000 руб"),
            Some((Some(250_000), Some(350_000)))
        );
        assert_eq!(sal("Вилка 200k-300k"), Some((Some(200_000), Some(300_000))));
        assert_eq!(sal("З/п до 300 000 ₽"), Some((None, Some(300_000))));
        assert_eq!(sal("от 180 тыс"), Some((Some(180_000), None)));
        assert_eq!(
            sal("Зарплата: 200-280"),
            Some((Some(200_000), Some(280_000)))
        );
        assert_eq!(
            sal("1,5 млн руб в месяц"),
            Some((Some(1_500_000), Some(1_500_000)))
        );
        assert_eq!(
            sal("300 000 ₽ на руки"),
            Some((Some(300_000), Some(300_000)))
        );
        assert_eq!(
            sal("Опыт от 3 лет. З/п до 250 000 ₽."),
            Some((None, Some(250_000)))
        );
    }

    #[test]
    fn salary_ignores_other_numbers() {
        assert_eq!(sal("опыт от 3 лет"), None);
        assert_eq!(sal("10 000 пользователей в сутки"), None);
        assert_eq!(sal("2019 - 2021"), None);
    }

    #[test]
    fn experience_statements() {
        assert_eq!(
            experience_total(&fold("Опыт работы — 4 года 3 месяца")),
            Some(4.3)
        );
        assert_eq!(experience_total(&fold("Опыт работы: 8 месяцев")), Some(0.7));
        assert_eq!(experience_total(&fold("Опыт работы с Kafka")), None);
        assert_eq!(
            experience_mention(&fold("3+ years of experience")),
            Some(3.0)
        );
        assert_eq!(
            experience_mention(&fold("опыт коммерческой разработки на Go от 5 лет")),
            Some(5.0)
        );
        assert_eq!(
            experience_mention(&fold("1,5 года коммерческого опыта")),
            Some(1.5)
        );
        assert_eq!(experience_mention(&fold("можно без опыта")), Some(0.0));
        assert_eq!(experience_mention(&fold("компания на рынке 10 лет")), None);
    }

    #[test]
    fn employment_ranges_merge_and_use_now() {
        // 2026-10-05
        let now = 1_791_158_400;
        let lines = [
            "январь 2021 - настоящее время",
            "сентябрь 2019 - декабрь 2020",
        ];
        // Jan 2021..Oct 2026 = 70 months, Sep 2019..Dec 2020 = 16 months.
        assert_eq!(employment_months(lines, now), Some(86));
        let overlapping = ["03.2020 - 12.2021", "06.2021 - 05.2022"];
        assert_eq!(employment_months(overlapping, now), Some(27));
        assert_eq!(employment_months(["2018 - 2022"], now), Some(48));
        assert_eq!(employment_months(["jun 2022 - present"], now), Some(53));
        assert_eq!(employment_months(["нет дат"], now), None);
    }

    #[test]
    fn specialization_from_title_and_skills() {
        let ids = skills::extract_ids("Go, PostgreSQL, Kafka");
        let g = guess_specialization("Senior Backend Developer (Go)", "", &ids).unwrap();
        assert_eq!(g.specialization, Specialization::Backend);
        assert!(g.confidence > 0.8, "{g:?}");
        let ids = skills::extract_ids("React, TypeScript");
        let g = guess_specialization("", "нужен человек", &ids).unwrap();
        assert_eq!(g.specialization, Specialization::Frontend);
        assert!(guess_specialization("", "ничего", &[]).is_none());
    }
}
