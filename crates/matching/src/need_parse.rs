//! Free-text vacancy / hiring need → structured draft (E-2: "можно свободным текстом").

use catalog::{Grade, Industry, WorkFormat};
use serde::{Deserialize, Serialize};

use crate::extract::{self, SpecializationGuess};
use crate::lexicon;
use crate::scoring::Need;
use crate::skills::{self, SkillId};
use crate::text::{self, fold};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ParsedNeed {
    pub specialization: Option<SpecializationGuess>,
    /// Primary grade: the first one in the title, else in the text, else from experience.
    pub grade: Option<Grade>,
    /// All grades mentioned ("middle/senior").
    pub grades: Vec<Grade>,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub work_formats: Vec<WorkFormat>,
    pub city: Option<String>,
    pub industries: Vec<Industry>,
    pub experience_years_min: Option<f32>,
}

impl ParsedNeed {
    /// Draft [`Need`]; `None` until the specialization is known.
    pub fn into_need(self, description: &str) -> Option<Need> {
        let grades = if self.grades.is_empty() {
            self.grade.into_iter().collect()
        } else {
            self.grades
        };
        Some(Need {
            specialization: self.specialization?.specialization,
            grades,
            allow_adjacent_grades: false,
            must_skills: self.must_skills,
            nice_skills: self.nice_skills,
            salary_min: self.salary_min.unwrap_or(0),
            salary_max: self.salary_max.unwrap_or(0),
            work_formats: self.work_formats,
            city: self.city,
            industries: self.industries,
            description: description.trim().to_owned(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Must,
    Nice,
}

/// Splits `Header: content`; a header is a short lead-in before a colon.
fn split_header(line: &str) -> (Option<&str>, &str) {
    if let Some((head, rest)) = line.split_once(':') {
        let words = head.split_whitespace().count();
        if (1..=4).contains(&words) && head.chars().count() <= 40 {
            return (Some(head), rest);
        }
    }
    (None, line)
}

fn sentences(s: &str) -> impl Iterator<Item = &str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    for (i, c) in s.char_indices() {
        let end_of_sentence = match c {
            ';' | '!' | '?' => true,
            '.' => bytes.get(i + 1).is_none_or(|b| b.is_ascii_whitespace()),
            _ => false,
        };
        if end_of_sentence {
            out.push(&s[start..i]);
            start = i + c.len_utf8();
        }
    }
    out.push(&s[start..]);
    out.into_iter().filter(|x| !x.trim().is_empty())
}

fn strip_bullet(line: &str) -> &str {
    line.trim_start_matches(|c: char| {
        c.is_whitespace() || matches!(c, '-' | '–' | '—' | '•' | '*' | '·' | '▪')
    })
}

/// Skills go to "nice" when they follow a "Будет плюсом:" header or share a clause with a
/// nice-to-have marker ("опыт с ClickHouse будет плюсом"). Inside a sentence a marker also
/// covers the following clauses until a must marker ("React, TypeScript, будет плюсом Next.js,
/// Storybook" → nice: Next.js, Storybook); everything else is required.
pub fn parse_need(text: &str) -> ParsedNeed {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let title = lines.first().copied().unwrap_or("");
    let body = lines
        .get(1..)
        .map_or_else(String::new, |rest| rest.join("\n"));

    let mut must: Vec<SkillId> = Vec::new();
    let mut nice: Vec<SkillId> = Vec::new();
    let mut mode = Mode::Must;
    for line in &lines {
        let (header, content) = split_header(strip_bullet(line));
        if let Some(h) = header {
            let named = skills::extract_ids(h);
            if lexicon::NICE_MATCHER.contains(h) {
                mode = Mode::Nice;
            } else if named.is_empty() || lexicon::MUST_MATCHER.contains(h) {
                mode = Mode::Must;
            }
            // "PostgreSQL: схемы, индексы" — the lead-in is the skill itself, inside whatever
            // section it is listed in.
            let target = if mode == Mode::Nice {
                &mut nice
            } else {
                &mut must
            };
            for id in named {
                if !target.contains(&id) {
                    target.push(id);
                }
            }
        } else if lexicon::NICE_MATCHER.contains(content)
            && content.split_whitespace().count() <= 3
            && skills::extract_ids(content).is_empty()
        {
            mode = Mode::Nice;
            continue;
        }
        for sentence in sentences(content) {
            let mut clause_nice = mode == Mode::Nice;
            for clause in sentence.split([',', ';']) {
                if lexicon::NICE_MATCHER.contains(clause) {
                    clause_nice = true;
                } else if lexicon::MUST_MATCHER.contains(clause) {
                    clause_nice = mode == Mode::Nice;
                }
                let target = if clause_nice { &mut nice } else { &mut must };
                for id in skills::extract_ids(clause) {
                    if !target.contains(&id) {
                        target.push(id);
                    }
                }
            }
        }
    }
    nice.retain(|id| !must.contains(id));

    let all_skills: Vec<SkillId> = text::distinct(must.iter().chain(&nice).copied());
    let specialization = extract::guess_specialization(title, &body, &all_skills);

    let folded = fold(text);
    let experience_years_min = extract::experience_mention(&folded);
    let mut grades = lexicon::grades(title);
    if grades.is_empty() {
        grades = lexicon::grades(text);
    }
    let grade = grades
        .first()
        .copied()
        .or_else(|| experience_years_min.map(extract::grade_for_experience));
    let salary = extract::salary(&folded).unwrap_or_default();

    let slugs = |ids: &[SkillId]| -> Vec<String> {
        ids.iter()
            .filter_map(|id| id.def().map(|d| d.slug.to_owned()))
            .collect()
    };
    ParsedNeed {
        specialization,
        grade,
        grades,
        must_skills: slugs(&must),
        nice_skills: slugs(&nice),
        salary_min: salary.min,
        salary_max: salary.max,
        work_formats: lexicon::work_formats(text),
        city: lexicon::canonical_city(text).map(str::to_owned),
        industries: lexicon::industries(text),
        experience_years_min,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::Specialization;

    #[test]
    fn header_and_sentence_markers() {
        let p = parse_need(
            "Нужно: Python, Django\nPostgreSQL обязательно. Опыт с Redis будет плюсом.\nЖелательно:\n- Kafka",
        );
        assert_eq!(p.must_skills, vec!["python", "django", "postgresql"]);
        assert_eq!(p.nice_skills, vec!["redis", "kafka"]);

        let p = parse_need(
            "Ищем синьор фронтендера на React + TypeScript, будет плюсом Next.js. Удалённо",
        );
        assert_eq!(p.must_skills, vec!["react", "typescript"]);
        assert_eq!(p.nice_skills, vec!["nextjs"]);

        let p = parse_need("Senior Go\nЗнание Kafka обязательно, ClickHouse желательно");
        assert_eq!(p.must_skills, vec!["go", "kafka"]);
        assert_eq!(p.nice_skills, vec!["clickhouse"]);
    }

    #[test]
    fn skill_named_in_a_lead_in() {
        let p = parse_need(
            "Требования:\n- Go от 2 лет;\n- PostgreSQL: схемы, индексы;\n- Kafka: консьюмеры.\nБудет плюсом:\n- ClickHouse: аналитика;\n- gRPC",
        );
        assert_eq!(p.must_skills, vec!["go", "postgresql", "kafka"]);
        assert_eq!(p.nice_skills, vec!["clickhouse", "grpc"]);
        let p = parse_need("Будет плюсом: Redis\nОбязательно: Go, PostgreSQL");
        assert_eq!(p.must_skills, vec!["go", "postgresql"]);
        assert_eq!(p.nice_skills, vec!["redis"]);
    }

    #[test]
    fn english_need() {
        let p = parse_need(
            "Middle ML Engineer (NLP)\nPython, PyTorch, transformers. Docker is a plus.\nRemote, 250k-300k RUB",
        );
        let spec = p.specialization.unwrap();
        assert_eq!(spec.specialization, Specialization::Ml);
        assert_eq!(p.grade, Some(Grade::Middle));
        assert_eq!(
            p.must_skills,
            vec!["machine_learning", "nlp", "python", "pytorch"]
        );
        assert_eq!(p.nice_skills, vec!["docker"]);
        assert_eq!((p.salary_min, p.salary_max), (Some(250_000), Some(300_000)));
        assert_eq!(p.work_formats, vec![WorkFormat::Remote]);
    }

    #[test]
    fn grade_from_experience_when_not_stated() {
        let p = parse_need("Ищем бэкенд-разработчика на Java, опыт от 5 лет");
        assert_eq!(p.grade, Some(Grade::Senior));
        assert!(p.grades.is_empty());
        assert_eq!(p.experience_years_min, Some(5.0));
    }

    #[test]
    fn into_need_requires_specialization() {
        assert!(parse_need("что-то непонятное").into_need("x").is_none());
        let n = parse_need("Senior Go developer, до 400 000 ₽")
            .into_need("Senior Go developer")
            .unwrap();
        assert_eq!(n.specialization, Specialization::Backend);
        assert_eq!(n.grades, vec![Grade::Senior]);
        assert_eq!((n.salary_min, n.salary_max), (0, 400_000));
    }
}
