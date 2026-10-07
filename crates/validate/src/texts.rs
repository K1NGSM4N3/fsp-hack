//! Synthetic profile and need texts for the matching validation (docs/validation/matching.md).
//!
//! What a person writes about their work follows what they really did: each job description
//! retells tasks of the person's real skills, mostly without naming the technology the way a
//! vacancy names it. The skill list in «о себе» is the declared one, with the same inflation as the
//! declared skills. A vacancy describes the same work in its own words. Domains of both sides are
//! drawn independently, so they are noise for any text method. Level cues follow the real ability,
//! noisily. These are assumptions of the generator, not facts about the market.

use catalog::{Grade, Specialization};
use matching::corpus::{SkillText, phrases};
use rand::{Rng, seq::IndexedRandom};
use rand_chacha::ChaCha8Rng;

fn role(spec: Specialization) -> &'static str {
    match spec {
        Specialization::Backend => "Бэкенд-разработчик",
        Specialization::Frontend => "Фронтенд-разработчик",
        Specialization::Devops => "DevOps-инженер",
        Specialization::Qa => "QA-инженер",
        Specialization::Ml => "ML-инженер",
        Specialization::Security => "Специалист по информационной безопасности",
    }
}

const DOMAINS: &[(&str, &str)] = &[
    ("финтех", "Платёжный сервис для малого бизнеса"),
    ("e-commerce", "Маркетплейс товаров для дома"),
    ("логистика", "Система управления доставками"),
    ("медтех", "Сервис записи в клиники"),
    ("госсектор", "Портал государственных услуг региона"),
    ("игры", "Мобильная игра с миллионом игроков"),
    ("телеком", "Биллинг оператора связи"),
    ("edtech", "Платформа онлайн-курсов"),
];

const FILLER: &[&str] = &[
    "Работал в продуктовой команде по скраму",
    "Участвовал в планировании и оценке задач",
    "Разбирал инциденты и писал постмортемы",
    "Общался с заказчиками и уточнял требования",
    "Писал техническую документацию",
];

const SENIOR_CUES: &[&str] = &[
    "Отвечал за архитектуру сервиса и технические решения",
    "Руководил командой из нескольких разработчиков и проводил собеседования",
    "Наставлял младших коллег и задавал стандарты кода",
];

const JUNIOR_CUES: &[&str] = &[
    "Пришёл после курсов и стажировки",
    "Делал задачи под руководством наставника",
    "Учусь и разбираюсь в новых для себя технологиях",
];

fn level_cue(rng: &mut ChaCha8Rng, theta: f64) -> Option<&'static str> {
    if !rng.random_bool(0.5) {
        return None;
    }
    if theta >= Grade::Senior.theta_cut() {
        SENIOR_CUES.choose(rng).copied()
    } else if theta < Grade::Middle.theta_cut() {
        JUNIOR_CUES.choose(rng).copied()
    } else {
        None
    }
}

fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn years_word(n: i64) -> &'static str {
    match (n % 10, n % 100) {
        (_, 11..=14) => "лет",
        (1, _) => "год",
        (2..=4, _) => "года",
        _ => "лет",
    }
}

/// «О себе» and job descriptions of one synthetic candidate, joined the way the API embeds a
/// profile ([`matching::semantic::candidate_text`]).
pub fn profile(
    rng: &mut ChaCha8Rng,
    spec: Specialization,
    real: &[&str],
    declared: &[&str],
    theta: f64,
    years: f64,
) -> String {
    let n = years.round().max(1.0) as i64;
    let (domain, _) = *DOMAINS.choose(rng).unwrap_or(&DOMAINS[0]);
    let stack: Vec<&str> = declared
        .iter()
        .filter_map(|s| matching::lookup(s).map(|d| d.name))
        .take(5)
        .collect();
    let mut about = format!(
        "{} с опытом {n} {}, последние проекты — {domain}.",
        role(spec),
        years_word(n)
    );
    if !stack.is_empty() {
        about.push_str(&format!(" Стек: {}.", stack.join(", ")));
    }
    let described: Vec<&SkillText> = real
        .iter()
        .filter(|_| rng.random_bool(0.75))
        .filter_map(|s| phrases(s))
        .collect();
    let jobs = (1 + (years / 3.0) as usize).min(3);
    let mut experience: Vec<(String, String)> = Vec::with_capacity(jobs);
    for j in 0..jobs {
        let mut parts: Vec<&str> = described
            .iter()
            .skip(j)
            .step_by(jobs)
            .filter_map(|s| s.work.choose(rng).copied())
            .collect();
        if let Some(f) = FILLER.choose(rng) {
            parts.push(f);
        }
        if j == 0
            && let Some(cue) = level_cue(rng, theta)
        {
            parts.push(cue);
        }
        let (_, project) = *DOMAINS.choose(rng).unwrap_or(&DOMAINS[0]);
        experience.push((
            role(spec).to_owned(),
            format!("{project}. {}.", parts.join(". ")),
        ));
    }
    matching::semantic::candidate_text(
        &[role(spec).to_owned()],
        &about,
        experience.iter().map(|(p, d)| (p.as_str(), d.as_str())),
    )
}

/// Title and description of a synthetic need.
pub fn need(
    rng: &mut ChaCha8Rng,
    spec: Specialization,
    grade: Grade,
    must: &[String],
    nice: &[String],
) -> (String, String) {
    let (_, project) = *DOMAINS.choose(rng).unwrap_or(&DOMAINS[0]);
    let title = format!("{} ({})", role(spec), grade.name_ru());
    let ask = |rng: &mut ChaCha8Rng, list: &[String]| -> Vec<&'static str> {
        list.iter()
            .filter_map(|s| phrases(s))
            .filter_map(|s| s.ask.choose(rng).copied())
            .collect()
    };
    let tasks = ask(rng, must);
    let plus = ask(rng, nice);
    let mut text = format!(
        "{project}: ищем специалиста в команду. Задачи и требования: {}.",
        lower_first(&tasks.join("; "))
    );
    if !plus.is_empty() {
        text.push_str(&format!(
            " Будет плюсом: {}.",
            lower_first(&plus.join("; "))
        ));
    }
    text.push_str(match grade {
        Grade::Junior => " Возьмём начинающего, дадим наставника.",
        Grade::Middle => " Ждём самостоятельности в задачах.",
        Grade::Senior => " Ждём опыта архитектурных решений и наставничества.",
    });
    (title, text)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn texts_are_deterministic_and_carry_the_parts() {
        let make = || {
            let mut rng = ChaCha8Rng::seed_from_u64(5);
            profile(
                &mut rng,
                Specialization::Backend,
                &["go", "kafka", "postgresql"],
                &["go", "kafka", "java"],
                1.4,
                6.0,
            )
        };
        let p = make();
        assert_eq!(p, make());
        assert!(p.starts_with("Бэкенд-разработчик\nБэкенд-разработчик с опытом 6 лет"));
        assert!(p.contains("Стек: Go, Apache Kafka, Java."), "{p}");
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let (title, text) = need(
            &mut rng,
            Specialization::Qa,
            Grade::Junior,
            &["manual_testing".into()],
            &[],
        );
        assert_eq!(title, "QA-инженер (Junior)");
        assert!(text.contains("Возьмём начинающего"));
        assert!(!text.contains("Будет плюсом"));
    }

    #[test]
    fn years_are_declined() {
        let words: Vec<&str> = [1, 2, 5, 11, 21, 24].map(years_word).to_vec();
        assert_eq!(words, ["год", "года", "лет", "лет", "год", "года"]);
    }
}
