//! Plain resume text (extracted from a PDF) → profile draft (C-1). Rules and dictionaries
//! only: the draft is shown to the candidate for confirmation, never trusted blindly.

use std::sync::LazyLock;

use catalog::{Grade, Industry, WorkFormat};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::extract::{self, SpecializationGuess};
use crate::lexicon;
use crate::skills;
use crate::text::{Matcher, fold};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ParsedResume {
    pub full_name: Option<String>,
    pub email: Option<String>,
    /// E.164, e.g. `+79161234567`.
    pub phone: Option<String>,
    /// `@handle`
    pub telegram: Option<String>,
    pub city: Option<String>,
    pub relocation: Option<bool>,
    /// Canonical skill slugs.
    pub skills: Vec<String>,
    pub specialization_hint: Option<SpecializationGuess>,
    /// Hint for the survey only — the category always comes from the test (M-1).
    pub grade_hint: Option<Grade>,
    pub experience_years: Option<f32>,
    pub roles: Vec<String>,
    pub soft_skills: Vec<String>,
    pub industries: Vec<Industry>,
    pub work_formats: Vec<WorkFormat>,
    pub salary_expectation: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Header,
    Experience,
    Education,
    Skills,
    About,
    Other,
}

#[rustfmt::skip]
const SECTION_HEADERS: &[(&str, Section)] = &[
    ("опыт работы", Section::Experience), ("опыт", Section::Experience), ("work experience", Section::Experience),
    ("professional experience", Section::Experience), ("employment history", Section::Experience),
    ("experience", Section::Experience), ("места работы", Section::Experience), ("трудовая деятельность", Section::Experience),
    ("образование", Section::Education), ("education", Section::Education), ("курсы", Section::Education),
    ("повышение квалификации", Section::Education), ("сертификаты", Section::Education),
    ("certifications", Section::Education), ("courses", Section::Education),
    ("ключевые навыки", Section::Skills), ("навыки", Section::Skills), ("hard skills", Section::Skills),
    ("soft skills", Section::Skills), ("skills", Section::Skills), ("технические навыки", Section::Skills),
    ("стек", Section::Skills), ("технологии", Section::Skills), ("tech stack", Section::Skills),
    ("личные качества", Section::Skills),
    ("о себе", Section::About), ("обо мне", Section::About), ("about me", Section::About), ("summary", Section::About),
    ("дополнительная информация", Section::About), ("profile", Section::About),
    ("проекты", Section::Other), ("projects", Section::Other), ("достижения", Section::Other),
    ("achievements", Section::Other), ("языки", Section::Other), ("languages", Section::Other),
    ("контакты", Section::Other), ("contacts", Section::Other), ("хобби", Section::Other), ("интересы", Section::Other),
];

const BULLETS: &[char] = &['-', '–', '—', '•', '*', '·', '▪', '●'];

fn strip_bullet(line: &str) -> &str {
    line.trim_start_matches(|c: char| c.is_whitespace() || BULLETS.contains(&c))
}

/// A header is a known title alone on its line, or followed by `:`/`-` or a number
/// ("Опыт работы — 6 лет"); "Опыт работы с Kafka" is prose.
fn section_of(folded_line: &str) -> Option<Section> {
    let line = strip_bullet(folded_line).trim();
    if line.chars().count() > 60 {
        return None;
    }
    SECTION_HEADERS.iter().find_map(|&(title, section)| {
        let rest = line.strip_prefix(title)?;
        let after = rest.trim_start();
        let is_header = rest.is_empty()
            || after.starts_with([':', '-'])
            || (rest.starts_with(' ') && after.starts_with(|c: char| c.is_ascii_digit()));
        is_header.then_some(section)
    })
}

#[rustfmt::skip]
const ROLE_WORDS: &[&str] = &[
    "разработчик=", "разработчица=", "программист=", "тестировщик=", "тестировщица=", "инженер=",
    "аналитик=", "архитектор=", "тимлид=", "техлид=", "руководитель=", "стажер=", "стажерка=",
    "администратор=", "специалист=", "консультант=", "верстальщик=", "пентестер=",
    "developer", "engineer", "programmer", "analyst", "architect", "scientist", "tester", "devops",
    "sre", "lead", "head", "intern", "administrator", "manager",
];

static ROLE_MATCHER: LazyLock<Matcher<()>> =
    LazyLock::new(|| Matcher::new(ROLE_WORDS.iter().map(|w| (*w, ()))));

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex is valid")
}

static EMAIL: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b"));
static PHONE: LazyLock<Regex> = LazyLock::new(|| re(r"\+?\d[\d\s\-()]{8,18}\d"));
static TG_LINK: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)(?:t\.me|telegram\.me)/([a-z0-9_]{4,32})"));
static TG_LABEL: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?i)(?:telegram|телеграмм?|tg|тг)(?:\s*:\s*@?|\s*-\s*@?|\s+@)([a-z0-9_]{4,32})\b")
});
static TG_BARE: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?:^|[\s,;|·(])@([A-Za-z][A-Za-z0-9_]{3,31})\b"));
static NAME_LABEL: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)^\s*(?:фио|ф\.\s?и\.\s?о\.|имя|full\s+name|name)\s*[:\-–—]\s*(.+)$"));
static PARENS: LazyLock<Regex> = LazyLock::new(|| re(r"\([^)]*\)"));

const CITY_LABELS: &[&str] = &[
    "проживает",
    "город",
    "место жительства",
    "местоположение",
    "location",
    "city",
];
const NOT_NAME: &[&str] = &[
    "резюме",
    "resume",
    "cv",
    "curriculum",
    "vitae",
    "мужчина",
    "женщина",
];
const HEADER_FALLBACK_LINES: usize = 12;
const MAX_ROLES: usize = 6;

fn normalize_phone(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    match digits.len() {
        11 if digits.starts_with(['7', '8']) => Some(format!("+7{}", &digits[1..])),
        10 if digits.starts_with('9') => Some(format!("+7{digits}")),
        11..=15 if raw.starts_with('+') => Some(format!("+{digits}")),
        _ => None,
    }
}

fn title_case(word: &str) -> String {
    let lower = word.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut upper_next = true;
    for c in lower.chars() {
        if upper_next {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
        upper_next = c == '-';
    }
    out
}

fn is_name_word(w: &str) -> bool {
    let mut chars = w.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let rest: Vec<char> = chars.collect();
    first.is_uppercase()
        && w.chars().all(|c| c.is_alphabetic() || c == '-')
        && (rest.iter().all(|c| !c.is_uppercase()) || rest.iter().all(|c| !c.is_lowercase()))
}

fn name_from_line(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    if !(2..=4).contains(&words.len()) || !words.iter().all(|w| is_name_word(w)) {
        return None;
    }
    let folded = fold(line);
    let looks_like_other = NOT_NAME
        .iter()
        .any(|w| folded.split_whitespace().any(|x| x == *w))
        || ROLE_MATCHER.contains(line)
        || !skills::extract_ids(line).is_empty()
        || lexicon::canonical_city(line).is_some();
    (!looks_like_other).then(|| {
        words
            .iter()
            .map(|w| title_case(w))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn full_name(lines: &[&str]) -> Option<String> {
    lines
        .iter()
        .take(15)
        .find_map(|l| NAME_LABEL.captures(l).and_then(|c| c.get(1)))
        .map(|m| {
            m.as_str()
                .split_whitespace()
                .map(title_case)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .or_else(|| lines.iter().take(8).find_map(|l| name_from_line(l)))
}

fn telegram(text: &str) -> Option<String> {
    [&*TG_LINK, &*TG_LABEL, &*TG_BARE]
        .iter()
        .find_map(|r| r.captures(text).and_then(|c| c.get(1)))
        .map(|m| format!("@{}", m.as_str()))
}

/// Job-title-like fragments: short segments with a role noun in nominative form, so
/// "менторство junior-разработчиков" is not mistaken for a title. Bare one-word segments
/// from lists ("Программист, разработчик") are hh.ru specialization tags, not titles.
fn roles<'a>(lines: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let line = PARENS.replace_all(strip_bullet(line), "");
        let segments: Vec<&str> = line
            .split([',', '|', '·', ';', '•'])
            .flat_map(|s| {
                s.split(" — ")
                    .flat_map(|s| s.split(" – "))
                    .flat_map(|s| s.split(" - "))
            })
            .map(|s| {
                s.trim_matches(|c: char| {
                    c.is_whitespace() || matches!(c, '.' | ':' | '"' | '«' | '»')
                })
            })
            .filter(|s| !s.is_empty())
            .collect();
        let single = segments.len() == 1;
        for seg in &segments {
            let words = seg.split_whitespace().count();
            let has_year = seg
                .as_bytes()
                .windows(4)
                .any(|w| w.iter().all(u8::is_ascii_digit));
            if words == 0
                || words > 6
                || seg.chars().count() > 60
                || seg.contains(['@', ':'])
                || has_year
                || (words == 1 && !single && !seg.contains('-'))
                || !ROLE_MATCHER.contains(seg)
            {
                continue;
            }
            let mut chars = seg.chars();
            let role: String = chars
                .next()
                .map(|c| c.to_uppercase().chain(chars).collect())
                .unwrap_or_default();
            if !out.iter().any(|r| fold(r) == fold(&role)) {
                out.push(role);
            }
            if out.len() >= MAX_ROLES {
                return out;
            }
        }
    }
    out
}

/// `now_unix` resolves "по настоящее время" in employment ranges.
pub fn parse_resume(text: &str, now_unix: i64) -> ParsedResume {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let folded: Vec<String> = lines.iter().map(|l| fold(l)).collect();
    let mut current = Section::Header;
    let sections: Vec<Section> = folded
        .iter()
        .map(|l| {
            if let Some(s) = section_of(l) {
                current = s;
            }
            current
        })
        .collect();
    let header_end = sections
        .iter()
        .position(|s| *s == Section::Experience)
        .unwrap_or(lines.len().min(HEADER_FALLBACK_LINES));
    let in_header =
        |i: usize| i < header_end && matches!(sections[i], Section::Header | Section::About);
    let header_text = (0..lines.len())
        .filter(|&i| in_header(i))
        .map(|i| lines[i])
        .collect::<Vec<_>>()
        .join("\n");
    let rest_text = (0..lines.len())
        .filter(|&i| !in_header(i))
        .map(|i| lines[i])
        .collect::<Vec<_>>()
        .join("\n");
    let folded_all = fold(text);

    let has_experience_section = sections.contains(&Section::Experience);
    let employment_lines = folded.iter().zip(&sections).filter_map(|(l, s)| {
        let counts = if has_experience_section {
            *s == Section::Experience
        } else {
            *s != Section::Education && !l.contains("университет") && !l.contains("university")
        };
        counts.then_some(l.as_str())
    });
    let experience_years = extract::experience_total(&folded_all)
        .or_else(|| {
            extract::employment_months(employment_lines, now_unix)
                .map(|m| (m as f32 / 12.0 * 10.0).round() / 10.0)
        })
        .or_else(|| extract::experience_mention(&folded_all));

    let skill_ids = skills::extract_ids(text);
    let grade_hint = lexicon::grades(&header_text)
        .first()
        .copied()
        .or_else(|| experience_years.map(extract::grade_for_experience));

    let city = lines
        .iter()
        .zip(&folded)
        .filter(|(_, f)| CITY_LABELS.iter().any(|l| f.contains(l)))
        .find_map(|(l, _)| lexicon::canonical_city(l))
        .or_else(|| {
            lines[..header_end]
                .iter()
                .find_map(|l| lexicon::canonical_city(l))
        })
        .map(str::to_owned);

    let role_lines = (0..lines.len())
        .filter(|&i| matches!(sections[i], Section::Header | Section::Experience))
        .map(|i| lines[i]);

    ParsedResume {
        full_name: full_name(&lines),
        email: EMAIL.find(text).map(|m| m.as_str().to_lowercase()),
        phone: PHONE
            .find_iter(text)
            .find_map(|m| normalize_phone(m.as_str())),
        telegram: telegram(text),
        city,
        relocation: lexicon::relocation(text),
        skills: skill_ids
            .iter()
            .filter_map(|id| id.def().map(|d| d.slug.to_owned()))
            .collect(),
        specialization_hint: extract::guess_specialization(&header_text, &rest_text, &skill_ids),
        grade_hint,
        experience_years,
        roles: roles(role_lines),
        soft_skills: lexicon::soft_skills(text),
        industries: lexicon::industries(text),
        work_formats: lexicon::work_formats(&header_text),
        salary_expectation: extract::salary(&fold(&header_text)).and_then(|s| s.min.or(s.max)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phones_normalize_to_e164() {
        assert_eq!(
            normalize_phone("+7 (916) 123-45-67").as_deref(),
            Some("+79161234567")
        );
        assert_eq!(
            normalize_phone("8 913 000-11-22").as_deref(),
            Some("+79130001122")
        );
        assert_eq!(
            normalize_phone("9161234567").as_deref(),
            Some("+79161234567")
        );
        assert_eq!(
            normalize_phone("+375 29 123-45-67").as_deref(),
            Some("+375291234567")
        );
        assert_eq!(normalize_phone("2019 2021 2022"), None);
    }

    #[test]
    fn section_headers() {
        assert_eq!(
            section_of("опыт работы - 6 лет 4 месяца"),
            Some(Section::Experience)
        );
        assert_eq!(section_of("опыт"), Some(Section::Experience));
        assert_eq!(section_of("навыки: python, go"), Some(Section::Skills));
        assert_eq!(section_of("опыт работы с kafka"), None);
    }

    #[test]
    fn names() {
        assert_eq!(
            name_from_line("ИВАНОВ Иван").as_deref(),
            Some("Иванов Иван")
        );
        assert_eq!(name_from_line("Senior Python-разработчик"), None);
        assert_eq!(name_from_line("Москва Россия"), None);
        assert_eq!(
            full_name(&["Резюме", "ФИО: петров пётр"]).as_deref(),
            Some("Петров Пётр")
        );
    }

    #[test]
    fn telegram_variants() {
        assert_eq!(telegram("tg: @dev_ivan").as_deref(), Some("@dev_ivan"));
        assert_eq!(
            telegram("https://t.me/anna_front").as_deref(),
            Some("@anna_front")
        );
        assert_eq!(telegram("Telegram Mini Apps"), None);
        assert_eq!(telegram("mail: ivan@mail.ru"), None);
    }
}
