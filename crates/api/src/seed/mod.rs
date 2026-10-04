//! Deterministic demo data: companies with needs and vacancies, a few hundred categorized
//! candidates (some linked to FSP), demo logins. Runs once on an empty database.

mod corpus;

use catalog::{Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Duration, Utc};
use matching::Achievement;
use rand::{
    Rng, SeedableRng,
    seq::{IndexedRandom, SliceRandom},
};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    contacts::Contacts,
    crypto::{Crypto, normalize_email},
    error::ApiResult,
    routes::candidate_profile::ExperienceEntry,
};

pub const DEMO_PASSWORD: &str = "demo12345";
const CANDIDATES_PER_SPEC: usize = 40;
const CONSENT_PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const CONSENT_PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const CONSENT_TERMS: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);

#[derive(Deserialize)]
struct Participant {
    fsp_id: String,
    achievements: Vec<serde_json::Value>,
}

pub struct SeedReport {
    pub companies: usize,
    pub candidates: usize,
    pub fsp_linked: usize,
}

struct Account {
    id: Uuid,
    email: String,
}

async fn account(
    tx: &tokio_postgres::Transaction<'_>,
    crypto: &Crypto,
    hash: &str,
    email: &str,
    role: db::types::UserRole,
    at: DateTime<Utc>,
) -> ApiResult<Account> {
    let id = Uuid::now_v7();
    let email = normalize_email(email);
    db::queries::seed::seed_verified_user()
        .bind(
            tx,
            &id,
            &role,
            &crypto.encrypt_str(&email),
            &crypto.blind_index(&email),
            &hash,
            &at,
        )
        .await?;
    let docs: &[Uuid] = match role {
        db::types::UserRole::Candidate => &[CONSENT_PROCESSING, CONSENT_PUBLICATION, CONSENT_TERMS],
        _ => &[CONSENT_PROCESSING, CONSENT_TERMS],
    };
    for doc in docs {
        db::queries::consents::insert_user_consent()
            .bind(tx, &Uuid::now_v7(), &id, doc, &None::<&str>, &Some("seed"))
            .await?;
    }
    Ok(Account { id, email })
}

fn grade_band(grade: Grade) -> (f64, f64) {
    match grade {
        Grade::Junior => (-1.0, 0.0),
        Grade::Middle => (0.0, 1.0),
        Grade::Senior => (1.0, 2.3),
    }
}

fn salary_base(spec: Specialization, grade: Grade) -> i64 {
    let base = match grade {
        Grade::Junior => 110_000,
        Grade::Middle => 240_000,
        Grade::Senior => 380_000,
    };
    let k = match spec {
        Specialization::Ml | Specialization::Security => 1.08,
        Specialization::Devops => 1.05,
        Specialization::Qa => 0.82,
        _ => 1.0,
    };
    (base as f64 * k) as i64
}

fn role_name(spec: Specialization) -> &'static str {
    match spec {
        Specialization::Backend => "Бэкенд-разработчик",
        Specialization::Frontend => "Фронтенд-разработчик",
        Specialization::Devops => "DevOps-инженер",
        Specialization::Qa => "QA-инженер",
        Specialization::Ml => "ML-инженер",
        Specialization::Security => "Специалист по безопасности",
    }
}

fn focus(spec: Specialization) -> &'static [&'static str] {
    match spec {
        Specialization::Backend => &[
            "высоконагруженные API",
            "платёжные сервисы",
            "микросервисы и очереди",
            "оптимизация запросов к БД",
        ],
        Specialization::Frontend => &[
            "сложные SPA",
            "дизайн-системы",
            "производительность интерфейсов",
            "личные кабинеты",
        ],
        Specialization::Devops => &[
            "Kubernetes-платформы",
            "CI/CD",
            "наблюдаемость и SLO",
            "инфраструктура как код",
        ],
        Specialization::Qa => &[
            "тест-дизайн",
            "автоматизацию UI",
            "тестирование API",
            "нагрузочное тестирование",
        ],
        Specialization::Ml => &[
            "компьютерное зрение",
            "рекомендательные системы",
            "NLP",
            "прогнозирование временных рядов",
        ],
        Specialization::Security => &[
            "пентесты веб-приложений",
            "безопасную разработку",
            "мониторинг инцидентов",
            "CTF-задачи",
        ],
    }
}

const SOFT: &[&str] = &[
    "Коммуникабельность",
    "Менторство",
    "Самостоятельность",
    "Работа в команде",
    "Лидерство",
    "Аналитическое мышление",
];
const GENERAL: &[&str] = &[
    "git",
    "algorithms",
    "data_structures",
    "unit_testing",
    "linux",
];

fn dominant_spec(achievements: &[Achievement]) -> Specialization {
    use matching::Discipline;
    let mut counts = [0u32; 6];
    for a in achievements {
        counts[a.discipline as usize] += 1;
    }
    let top = (0..6).max_by_key(|&i| counts[i]).unwrap_or(0);
    match Discipline::ALL[top] {
        Discipline::Security => Specialization::Security,
        Discipline::Ai => Specialization::Ml,
        Discipline::Product => Specialization::Frontend,
        Discipline::Robotics | Discipline::Uav => Specialization::Devops,
        Discipline::Algorithmic => Specialization::Backend,
    }
}

pub async fn seed(
    pool: &deadpool_postgres::Pool,
    crypto: &Crypto,
) -> ApiResult<Option<SeedReport>> {
    let mut client = pool.get().await?;
    if db::queries::seed::demo_seeded().bind(&client).one().await? > 0 {
        return Ok(None);
    }
    let hash = crypto.passwords.hash(DEMO_PASSWORD)?;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let now = Utc::now();
    let tx = client.transaction().await?;

    let mut companies = Vec::with_capacity(corpus::COMPANIES.len());
    for (i, c) in corpus::COMPANIES.iter().enumerate() {
        let email = if i == 0 {
            "employer@demo.fsp".to_owned()
        } else {
            format!("hr{i}@demo.fsp")
        };
        let owner = account(
            &tx,
            crypto,
            &hash,
            &email,
            db::types::UserRole::Employer,
            now - Duration::days(120),
        )
        .await?;
        let id = Uuid::now_v7();
        db::queries::seed::seed_company()
            .bind(
                &tx,
                &id,
                &owner.id,
                &c.name,
                &c.description,
                &c.industry,
                &c.city,
                &c.size,
                &c.email,
                &c.telegram,
            )
            .await?;
        companies.push(id);
    }
    for v in corpus::VACANCIES {
        let company = companies[v.company];
        let skills: Vec<String> = v.skills.iter().map(|s| (*s).to_owned()).collect();
        let published = now - Duration::days(rng.random_range(1..30));
        db::queries::vacancies::insert_vacancy()
            .bind(
                &tx,
                &Uuid::now_v7(),
                &company,
                &v.title,
                &v.description,
                &v.specialization,
                &v.grade,
                &skills,
                &v.salary.0,
                &v.salary.1,
                &v.work_format,
                &Some(corpus::COMPANIES[v.company].city),
                &db::types::VacancyStatus::Published,
                &Some(published),
            )
            .await?;
        let split = skills.len().min(3);
        db::queries::needs::insert_need()
            .bind(
                &tx,
                &Uuid::now_v7(),
                &company,
                &v.title,
                &v.description,
                &v.specialization,
                &vec![v.grade.to_owned()],
                &true,
                &skills[..split].to_vec(),
                &skills[split..].to_vec(),
                &v.salary.0,
                &v.salary.1,
                &vec![v.work_format.to_owned()],
                &None::<&str>,
                &vec![corpus::COMPANIES[v.company].industry.to_owned()],
            )
            .await?;
    }

    let participants: Vec<Participant> =
        serde_json::from_str(include_str!("../../../fsp-mock/data/participants.json"))?;
    let mut fsp_queue: Vec<(String, Vec<Achievement>, serde_json::Value)> = participants
        .into_iter()
        .filter(|p| p.fsp_id.as_str() >= "FSP-000011")
        .map(|p| {
            let typed: Vec<Achievement> = p
                .achievements
                .iter()
                .filter_map(|a| serde_json::from_value(a.clone()).ok())
                .collect();
            (p.fsp_id, typed, serde_json::Value::Array(p.achievements))
        })
        .collect();

    let mut n = 0usize;
    let mut fsp_linked = 0usize;
    for spec in Specialization::ALL.iter().copied() {
        for k in 0..CANDIDATES_PER_SPEC {
            let grade = match rng.random_range(0..10) {
                0..=3 => Grade::Junior,
                4..=7 => Grade::Middle,
                _ => Grade::Senior,
            };
            let (lo, hi) = grade_band(grade);
            let theta = rng.random_range(lo..hi);
            let female = rng.random_bool(0.35);
            let first = if female {
                corpus::FEMALE_FIRST.choose(&mut rng)
            } else {
                corpus::MALE_FIRST.choose(&mut rng)
            }
            .copied()
            .unwrap_or("Алекс");
            let last_pair = corpus::LAST
                .choose(&mut rng)
                .copied()
                .unwrap_or(("Иванов", "Иванова"));
            let full_name = format!("{} {first}", if female { last_pair.1 } else { last_pair.0 });
            let email = if n == 0 {
                "candidate@demo.fsp".to_owned()
            } else {
                format!("cand{n}@demo.fsp")
            };
            let created = now - Duration::days(rng.random_range(30..200));
            let acc = account(
                &tx,
                crypto,
                &hash,
                &email,
                db::types::UserRole::Candidate,
                created,
            )
            .await?;

            let years: f32 = match grade {
                Grade::Junior => rng.random_range(0.5..2.0),
                Grade::Middle => rng.random_range(2.0..5.0),
                Grade::Senior => rng.random_range(5.0..12.0),
            };
            let mut typical: Vec<&str> = matching::typical_skills(spec).to_vec();
            typical.shuffle(&mut rng);
            let take = rng.random_range(4..=typical.len().min(8));
            let mut skills: Vec<(&str, &str)> = Vec::new();
            for (i, s) in typical[..take].iter().enumerate() {
                let source = if i < 3 && rng.random_bool(0.7) {
                    "test"
                } else if rng.random_bool(0.3) {
                    "resume"
                } else {
                    "declared"
                };
                skills.push((s, "declared"));
                if source != "declared" {
                    skills.push((s, source));
                }
            }
            if let Some(g) = GENERAL.choose(&mut rng) {
                skills.push((g, "declared"));
            }
            let skill_names: Vec<&str> = skills
                .iter()
                .filter(|(_, src)| *src == "declared")
                .map(|(s, _)| matching::lookup(s).map_or(*s, |d| d.name))
                .collect();
            let topic = focus(spec).choose(&mut rng).copied().unwrap_or("");
            let about = format!(
                "{} с опытом {:.0} {}. Занимаюсь направлением «{topic}». Основной стек: {}.",
                role_name(spec),
                years.round(),
                if years.round() as i64 % 10 == 1 && years.round() as i64 != 11 {
                    "год"
                } else if (2..=4).contains(&(years.round() as i64 % 10)) {
                    "года"
                } else {
                    "лет"
                },
                skill_names.join(", ")
            );
            let formats: Vec<String> = {
                let mut f: Vec<WorkFormat> = WorkFormat::ALL
                    .iter()
                    .copied()
                    .filter(|_| rng.random_bool(0.55))
                    .collect();
                if f.is_empty() {
                    f.push(WorkFormat::Remote);
                }
                f.iter().map(|x| x.slug().to_owned()).collect()
            };
            let industry_count = rng.random_range(1..=2);
            let industries: Vec<String> = Industry::ALL
                .choose_multiple(&mut rng, industry_count)
                .map(|i| i.slug().to_owned())
                .collect();
            let jobs = rng.random_range(1..=3);
            let experience: Vec<ExperienceEntry> = (0..jobs)
                .map(|j| {
                    let start_year = 2026 - (years as i32) + j * 2;
                    ExperienceEntry {
                        company: corpus::PAST_EMPLOYERS
                            .choose(&mut rng)
                            .copied()
                            .unwrap_or("Фриланс")
                            .to_owned(),
                        position: role_name(spec).to_owned(),
                        start: format!("{}-{:02}", start_year.min(2026), rng.random_range(1..=12)),
                        end: None,
                        description: String::new(),
                    }
                })
                .collect();
            let contacts = Contacts {
                phone: Some(format!("+79{:09}", rng.random_range(0..1_000_000_000u64))),
                telegram: Some(format!("@demo_{n}")),
                email: Some(acc.email.clone()),
            };
            let status = match rng.random_range(0..20) {
                0 => db::types::SearchStatus::Closed,
                1..=8 => db::types::SearchStatus::Active,
                _ => db::types::SearchStatus::Open,
            };
            let salary = (!rng.random_bool(0.1)).then(|| {
                let base = salary_base(spec, grade) as f64 * rng.random_range(0.85..1.2);
                (base / 10_000.0).round() as i64 * 10_000
            });
            let last_activity = now - Duration::hours(rng.random_range(1..24 * 90));
            db::queries::seed::seed_candidate()
                .bind(
                    &tx,
                    &acc.id,
                    &crate::auth::routes::public_code(acc.id),
                    &crypto.encrypt_str(&full_name),
                    &crypto.encrypt_json(&contacts)?,
                    &corpus::CITIES.choose(&mut rng).copied(),
                    &rng.random_bool(0.2),
                    &formats,
                    &salary,
                    &about,
                    &Some(years),
                    &vec![role_name(spec).to_owned()],
                    &SOFT
                        .choose_multiple(&mut rng, 2)
                        .map(|s| (*s).to_owned())
                        .collect::<Vec<_>>(),
                    &industries,
                    &serde_json::to_value(&experience)?,
                    &status,
                    &rng.random_bool(0.3),
                    &last_activity,
                    &created,
                )
                .await?;
            for (skill, source) in &skills {
                db::queries::candidates::insert_candidate_skill()
                    .bind(&tx, &acc.id, skill, source)
                    .await?;
            }
            let assigned = created + Duration::days(rng.random_range(1..10));
            db::queries::grades::insert_grade_assignment()
                .bind(
                    &tx,
                    &Uuid::now_v7(),
                    &acc.id,
                    &spec.slug(),
                    &grade.slug(),
                    &theta,
                    &0.3,
                    &None::<Uuid>,
                    &"placement",
                    &assigned,
                )
                .await?;
            db::queries::grades::set_candidate_category()
                .bind(&tx, &spec.slug(), &grade.slug(), &theta, &assigned, &acc.id)
                .await?;

            let fsp_pick = if n == 0 {
                fsp_queue.first().map(|_| 0)
            } else if k % 3 == 0 {
                fsp_queue
                    .iter()
                    .position(|(_, typed, _)| !typed.is_empty() && dominant_spec(typed) == spec)
            } else if k == 1 {
                fsp_queue.iter().position(|(_, typed, _)| typed.is_empty())
            } else {
                None
            };
            if let Some(i) = fsp_pick {
                let (fsp_id, _, raw) = fsp_queue.remove(i);
                db::queries::seed::seed_fsp_link()
                    .bind(&tx, &acc.id, &fsp_id, &raw)
                    .await?;
                fsp_linked += 1;
            }
            n += 1;
        }
    }
    tx.commit().await?;
    Ok(Some(SeedReport {
        companies: companies.len(),
        candidates: n,
        fsp_linked,
    }))
}
