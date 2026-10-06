//! Deterministic demo data: companies with needs and vacancies, a few hundred categorized
//! candidates (some linked to FSP, most with a proctored test), the moderation queues, demo
//! logins. Runs once on an empty database.

mod corpus;
mod demo;
mod moderation;

pub use moderation::{DEMO_DISPUTED_EMAIL, DEMO_MODERATOR_EMAIL};

use catalog::{Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Duration, Utc};
use matching::Achievement;
use rand::{
    Rng, SeedableRng,
    seq::{IndexedRandom, SliceRandom},
};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    contacts::Contacts,
    crypto::{Crypto, normalize_email},
    error::{ApiResult, InternalError},
    public_code,
    routes::candidate_profile::ExperienceEntry,
};

pub const DEMO_PASSWORD: &str = "demo12345";
pub const CANDIDATES_PER_SPEC: usize = 40;
const CONSENT_PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const CONSENT_PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const CONSENT_TERMS: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);
const CONSENT_VIDEO: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000004);
/// Separate stream for proctoring: adding it left every earlier draw, and so the rest of the demo
/// data, unchanged.
const PROCTORING_SEED: u64 = 4242;
/// Same for the 14–17-year-olds: their birth dates and representatives come from their own stream.
const MINORS_SEED: u64 = 1417;
const CONSENT_GUARDIAN: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000a01);
pub const DEMO_MINOR_EMAIL: &str = "minor@demo.fsp";

#[derive(Deserialize)]
struct Participant {
    fsp_id: String,
    achievements: Vec<serde_json::Value>,
}

pub struct SeedReport {
    pub companies: usize,
    pub candidates: usize,
    pub fsp_linked: usize,
    /// Candidates whose tests were taken with the camera / without it.
    pub with_video: usize,
    pub without_video: usize,
    /// Candidates whose tests carry behavioral remarks (tab leaves, pastes).
    pub with_remarks: usize,
    /// 14–17-year-olds whose representative confirmed the consent / has not answered yet.
    pub minors_confirmed: usize,
    pub minors_pending: usize,
    /// Tests waiting for the moderator: flagged with events, flagged without proctoring data,
    /// remarks only; flagged vacancies and open complaints.
    pub review_flagged: usize,
    pub review_unproctored: usize,
    pub review_remarks: usize,
    pub flagged_vacancies: usize,
    pub complaints: usize,
}

struct Account {
    id: Uuid,
    email: String,
}

async fn account(
    tx: &mut PgConnection,
    crypto: &Crypto,
    hash: &str,
    email: &str,
    role: db::types::UserRole,
    at: DateTime<Utc>,
) -> ApiResult<Account> {
    let id = Uuid::now_v7();
    let email = normalize_email(email);
    db::seed::seed_verified_user(
        &mut *tx,
        id,
        role,
        &crypto.encrypt_str(&email),
        &crypto.blind_index(&email),
        hash,
        at,
    )
    .await?;
    let docs: &[Uuid] = match role {
        db::types::UserRole::Candidate => &[CONSENT_PROCESSING, CONSENT_PUBLICATION, CONSENT_TERMS],
        _ => &[CONSENT_PROCESSING, CONSENT_TERMS],
    };
    for doc in docs {
        db::seed::seed_registration_consent(
            &mut *tx,
            Uuid::now_v7(),
            id,
            *doc,
            at,
            db::types::AgeBracket::Adult,
        )
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

struct SimulatedSession {
    state: assessment::SessionState,
    decision: assessment::Decision,
    responses: Vec<(assessment::ItemRef, bool, u32, f64, f64)>,
}

/// Runs the production CAT engine with responses drawn from the 3PL model at the candidate's
/// true ability, so demo histories look exactly like real ones.
fn simulate(
    rng: &mut ChaCha8Rng,
    spec: Specialization,
    claimed: Grade,
    theta: f64,
) -> Option<SimulatedSession> {
    let bank = assessment::bank();
    let mut state = assessment::start(spec, claimed, rng.random(), Vec::new());
    let mut responses = Vec::new();
    loop {
        if let Some(decision) = assessment::decision(&state) {
            return Some(SimulatedSession {
                state,
                decision,
                responses,
            });
        }
        let item = assessment::next_item(&state)?;
        let params = bank.params(&item)?;
        let correct = rng.random::<f64>() < assessment::irt::p_correct(theta, &params);
        let elapsed = rng.random_range(15_000..95_000);
        assessment::record(&mut state, item.clone(), correct, elapsed).ok()?;
        responses.push((
            item,
            correct,
            elapsed,
            state.summary.theta,
            state.summary.se,
        ));
    }
}

async fn store_session(
    tx: &mut PgConnection,
    candidate: Uuid,
    kind: &str,
    sim: &SimulatedSession,
    started: DateTime<Utc>,
) -> ApiResult<(Uuid, DateTime<Utc>)> {
    let id = Uuid::now_v7();
    let total_ms: i64 = sim.responses.iter().map(|r| i64::from(r.2)).sum();
    let finished = started + Duration::milliseconds(total_ms);
    let d = &sim.decision;
    let status = if d.passed {
        db::types::AssessmentStatus::Passed
    } else {
        db::types::AssessmentStatus::Failed
    };
    db::seed::seed_session(
        &mut *tx,
        &db::seed::SeedSession {
            id,
            candidate_id: candidate,
            specialization: sim.state.specialization.slug(),
            claimed_grade: sim.state.claimed_grade.slug(),
            attempt_kind: kind,
            status,
            state: &serde_json::to_value(&sim.state)?,
            theta: d.theta,
            se: d.se,
            measured_grade: d.grade_measured.map(Grade::slug),
            confident_above_next: d.confident_above_next,
            person_fit_lz: d.person_fit_lz,
            aberrant: d.aberrant,
            items_count: sim.responses.len() as i32,
            started_at: started,
            finished_at: finished,
        },
    )
    .await?;
    let mut at = started;
    for (seq, (item, correct, elapsed, theta, se)) in sim.responses.iter().enumerate() {
        at += Duration::milliseconds(i64::from(*elapsed));
        db::assessment::insert_response(
            &mut *tx,
            &db::assessment::NewResponse {
                session_id: id,
                seq: seq as i32,
                template_id: &item.template_id,
                level: i16::from(item.level),
                seed: item.seed as i64,
                answer: None,
                correct: *correct,
                timed_out: false,
                elapsed_ms: *elapsed as i32,
                theta_after: *theta,
                se_after: *se,
                answered_at: at,
            },
        )
        .await?;
    }
    Ok((id, finished))
}

struct StoredSession {
    id: Uuid,
    started: DateTime<Utc>,
    finished: DateTime<Utc>,
    items: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Camera {
    Recorded,
    Declined,
    /// The camera dropped mid-test: fewer chunks and a `camera_lost` event.
    Interrupted,
}

struct ProctoringOutcome {
    camera: Camera,
    remarks: bool,
}

fn at_offset(rng: &mut ChaCha8Rng, s: &StoredSession) -> DateTime<Utc> {
    let span = (s.finished - s.started).num_milliseconds().max(1);
    s.started + Duration::milliseconds(rng.random_range(0..span))
}

/// Demo proctoring for one candidate: the choice of camera, chunk metadata, integrity events.
/// No video files exist. Recordings past retention are marked purged, as the sweeper would have
/// done; recent ones keep `purged_at` empty and the playback endpoint answers
/// `recording_unavailable` because the files are missing.
async fn seed_proctoring(
    tx: &mut PgConnection,
    rng: &mut ChaCha8Rng,
    candidate: Uuid,
    sessions: &[StoredSession],
    now: DateTime<Utc>,
    retention_days: i64,
) -> ApiResult<Option<ProctoringOutcome>> {
    let Some(first) = sessions.first() else {
        return Ok(None);
    };
    let camera = match rng.random_range(0..100) {
        0..75 => Camera::Recorded,
        75..95 => Camera::Declined,
        _ => Camera::Interrupted,
    };
    let remarks = rng.random_bool(0.1);
    let consent_id = if camera == Camera::Declined {
        None
    } else {
        let grant = Uuid::now_v7();
        db::seed::seed_consent(&mut *tx, grant, candidate, CONSENT_VIDEO, first.started).await?;
        Some(grant)
    };
    let retention = Duration::days(retention_days);
    for s in sessions {
        let video = camera != Camera::Declined;
        db::seed::seed_proctoring(
            &mut *tx,
            &db::seed::SeedProctoring {
                session_id: s.id,
                candidate_id: candidate,
                camera: video,
                consent_id,
                started_at: s.started,
                purged_at: (video && s.started + retention <= now).then(|| s.started + retention),
            },
        )
        .await?;
        if video {
            let full = ((s.finished - s.started).num_seconds() / 10 + 1) as i32;
            let chunks = if camera == Camera::Interrupted {
                (full / 2).max(1)
            } else {
                full
            };
            db::seed::seed_chunks(
                &mut *tx,
                s.id,
                chunks,
                rng.random_range(90_000..140_000),
                s.started,
            )
            .await?;
            if camera == Camera::Interrupted {
                let event = db::proctoring::Event {
                    kind: db::types::IntegrityEvent::CameraLost,
                    occurred_at: s.started + Duration::seconds(i64::from(chunks) * 10),
                    item_seq: Some((s.items / 2) as i16),
                    duration_ms: None,
                };
                db::proctoring::insert_event(&mut *tx, s.id, &event).await?;
            }
        }
        if remarks {
            let leaves = rng.random_range(1..=3);
            let pastes = u8::from(rng.random_bool(0.3));
            for i in 0..leaves + pastes {
                let kind = if i >= leaves {
                    db::types::IntegrityEvent::Paste
                } else if rng.random_bool(0.7) {
                    db::types::IntegrityEvent::TabHidden
                } else {
                    db::types::IntegrityEvent::WindowBlur
                };
                let event = db::proctoring::Event {
                    kind,
                    occurred_at: at_offset(rng, s),
                    item_seq: Some(rng.random_range(0..s.items.max(1)) as i16),
                    duration_ms: (kind != db::types::IntegrityEvent::Paste)
                        .then(|| rng.random_range(2_000..45_000)),
                };
                db::proctoring::insert_event(&mut *tx, s.id, &event).await?;
            }
        }
    }
    Ok(Some(ProctoringOutcome { camera, remarks }))
}

/// A categorized junior who may become one of the demo 14–17-year-olds.
struct Junior {
    id: Uuid,
    spec: Specialization,
    created: DateTime<Utc>,
    surname: (&'static str, &'static str),
}

/// The first junior of every specialization registered as 14–17: a birth date, the declaration on
/// the registration grants, no work history. Two of six wait for the representative (one of them
/// logs in as `minor@demo.fsp`), the rest are confirmed and show the «14–17 лет» badge.
async fn seed_minors(
    tx: &mut PgConnection,
    crypto: &Crypto,
    rng: &mut ChaCha8Rng,
    juniors: &[Junior],
    now: DateTime<Utc>,
) -> ApiResult<(usize, usize)> {
    let today = now.date_naive();
    let (mut confirmed, mut pending) = (0usize, 0usize);
    let picks = Specialization::ALL
        .iter()
        .filter_map(|spec| juniors.iter().find(|j| j.spec == *spec));
    for (i, j) in picks.enumerate() {
        let days = rng.random_range(14 * 365 + 210..18 * 365 - 30);
        let birth = today
            .checked_sub_days(chrono::Days::new(days))
            .unwrap_or(today);
        db::seed::seed_minor(&mut *tx, j.id, birth).await?;

        let mother = rng.random_bool(0.7);
        let first = if mother {
            corpus::FEMALE_FIRST.choose(rng)
        } else {
            corpus::MALE_FIRST.choose(rng)
        }
        .copied()
        .unwrap_or("Алекс");
        let name = format!("{} {first}", if mother { j.surname.1 } else { j.surname.0 });
        let email = format!("parent{}@demo.fsp", i + 1);
        let waits = i % 3 == 2;
        let sent = if waits {
            now - Duration::hours(rng.random_range(6..72))
        } else {
            j.created + Duration::hours(rng.random_range(1..30))
        };
        let consent_id = if waits {
            None
        } else {
            let grant = Uuid::now_v7();
            let at = sent + Duration::minutes(rng.random_range(20..2_000));
            db::seed::seed_consent(&mut *tx, grant, j.id, CONSENT_GUARDIAN, at).await?;
            Some((grant, at))
        };
        let (_, confirm_hash) = crate::crypto::new_token();
        let (_, revoke_hash) = crate::crypto::new_token();
        db::seed::seed_guardian(
            &mut *tx,
            &db::seed::SeedGuardian {
                id: Uuid::now_v7(),
                candidate_id: j.id,
                full_name_enc: &crypto.encrypt_str(&name),
                email_enc: &crypto.encrypt_str(&email),
                email_bidx: &crypto.blind_index(&email),
                confirm_hash: &confirm_hash,
                revoke_hash: &revoke_hash,
                sent_at: sent,
                confirm_expires_at: sent + Duration::days(crate::guardian::CONFIRM_TTL_DAYS),
                consent_id: consent_id.map(|c| c.0),
                answered_at: consent_id.map(|c| c.1),
                created_at: j.created,
            },
        )
        .await?;
        if waits && pending == 0 {
            db::seed::seed_login_email(
                &mut *tx,
                j.id,
                &crypto.encrypt_str(DEMO_MINOR_EMAIL),
                &crypto.blind_index(DEMO_MINOR_EMAIL),
            )
            .await?;
        }
        if waits {
            pending += 1;
        } else {
            confirmed += 1;
        }
    }
    Ok((confirmed, pending))
}

pub async fn seed(
    pool: &PgPool,
    crypto: &Crypto,
    retention_days: i64,
) -> ApiResult<Option<SeedReport>> {
    seed_sized(pool, crypto, retention_days, CANDIDATES_PER_SPEC).await
}

/// A larger bank for load tests (`SEED_CANDIDATES_PER_SPEC`); the demo uses [`CANDIDATES_PER_SPEC`].
pub async fn seed_sized(
    pool: &PgPool,
    crypto: &Crypto,
    retention_days: i64,
    per_spec: usize,
) -> ApiResult<Option<SeedReport>> {
    if db::seed::demo_seeded(pool).await? > 0 {
        return Ok(None);
    }
    let hash = crypto.passwords.hash(DEMO_PASSWORD)?;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut proctoring_rng = ChaCha8Rng::seed_from_u64(PROCTORING_SEED);
    let mut minors_rng = ChaCha8Rng::seed_from_u64(MINORS_SEED);
    let mut moderation_rng = ChaCha8Rng::seed_from_u64(moderation::MODERATION_SEED);
    let mut juniors: Vec<Junior> = Vec::new();
    let mut reviewables: Vec<moderation::Reviewable> = Vec::new();
    let mut everyone: Vec<Uuid> = Vec::new();
    let now = Utc::now();
    let mut tx = pool.begin().await?;

    let mut companies = Vec::with_capacity(corpus::COMPANIES.len());
    for c in corpus::COMPANIES {
        let owner = account(
            &mut tx,
            crypto,
            &hash,
            c.owner,
            db::types::UserRole::Employer,
            now - Duration::days(120),
        )
        .await?;
        let id = Uuid::now_v7();
        db::seed::seed_company(
            &mut *tx,
            &db::seed::SeedCompany {
                id,
                owner_user_id: owner.id,
                name: c.name,
                website: c.website,
                description: c.description,
                industry: c.industry,
                city: c.city,
                size: c.size,
                contact_email: c.email,
                contact_telegram: c.telegram,
            },
        )
        .await?;
        companies.push(id);
    }
    for v in corpus::VACANCIES {
        let company = companies[v.company];
        let skills: Vec<String> = v.skills.iter().map(|s| (*s).to_owned()).collect();
        let published = now - Duration::days(rng.random_range(1..30));
        db::vacancies::insert_vacancy(
            &mut *tx,
            Uuid::now_v7(),
            company,
            db::types::VacancyStatus::Published,
            Some(published),
            &db::vacancies::VacancyFields {
                title: v.title,
                description: v.description,
                specialization: v.specialization,
                grade: v.grade,
                skills: &skills,
                salary_min: v.salary.0,
                salary_max: v.salary.1,
                work_format: v.work_format,
                city: Some(corpus::COMPANIES[v.company].city),
            },
        )
        .await?;
        if v.company == demo::DEMO_COMPANY {
            continue;
        }
        let split = skills.len().min(3);
        db::needs::insert_need(
            &mut *tx,
            Uuid::now_v7(),
            company,
            &db::needs::NeedFields {
                title: v.title,
                description: v.description,
                specialization: v.specialization,
                grades: &[v.grade.to_owned()],
                allow_adjacent_grades: true,
                must_skills: &skills[..split],
                nice_skills: &skills[split..],
                salary_min: v.salary.0,
                salary_max: v.salary.1,
                work_formats: &[v.work_format.to_owned()],
                city: None,
                industries: &[corpus::COMPANIES[v.company].industry.to_owned()],
            },
        )
        .await?;
    }

    demo::verify_domains(&mut tx, &companies).await?;
    demo::seed_demo_needs(&mut tx, companies[demo::DEMO_COMPANY], now).await?;

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
    let (mut with_video, mut without_video, mut with_remarks) = (0usize, 0usize, 0usize);
    for spec in Specialization::ALL.iter().copied() {
        for k in 0..per_spec {
            let target = match rng.random_range(0..10) {
                0..=3 => Grade::Junior,
                4..=7 => Grade::Middle,
                _ => Grade::Senior,
            };
            let (lo, hi) = grade_band(target);
            let true_theta = rng.random_range(lo - 0.25..hi);
            let claimed = match rng.random_range(0..10) {
                0..=1 => target.next().unwrap_or(target),
                2 => target.prev().unwrap_or(target),
                _ => target,
            };
            let grade = target;
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
                &mut tx,
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
            let candidate = db::seed::SeedCandidate {
                user_id: acc.id,
                public_code: "",
                full_name_enc: &crypto.encrypt_str(&full_name),
                contacts_enc: &crypto.encrypt_json(&contacts)?,
                city: corpus::CITIES.choose(&mut rng).copied(),
                relocation: rng.random_bool(0.2),
                work_formats: &formats,
                salary_expectation_min: salary,
                about: &about,
                experience_years: Some(years),
                roles: &[role_name(spec).to_owned()],
                soft_skills: &SOFT
                    .choose_multiple(&mut rng, 2)
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>(),
                industries: &industries,
                experience: &serde_json::to_value(&experience)?,
                search_status: status,
                show_name: rng.random_bool(0.3),
                last_activity_at: last_activity,
                created_at: created,
            };
            let mut seeded = false;
            for code in public_code::candidates(acc.id).take(public_code::ATTEMPTS) {
                let row = db::seed::SeedCandidate {
                    public_code: &code,
                    ..candidate
                };
                if db::seed::seed_candidate(&mut *tx, &row).await? == 1 {
                    seeded = true;
                    break;
                }
            }
            if !seeded {
                return Err(InternalError::PublicCodeExhausted(public_code::ATTEMPTS).into());
            }
            for (skill, source) in &skills {
                db::candidates::insert_candidate_skill(&mut *tx, acc.id, skill, source).await?;
            }
            let mut started = created + Duration::days(rng.random_range(1..10));
            let mut kind = "placement";
            let mut try_grade = claimed;
            let mut assigned: Option<(Grade, f64, f64, Uuid, DateTime<Utc>)> = None;
            let mut verified: Vec<&'static str> = Vec::new();
            let mut stored: Vec<StoredSession> = Vec::new();
            for _ in 0..3 {
                let Some(sim) = simulate(&mut rng, spec, try_grade, true_theta) else {
                    break;
                };
                let (session_id, finished) =
                    store_session(&mut tx, acc.id, kind, &sim, started).await?;
                stored.push(StoredSession {
                    id: session_id,
                    started,
                    finished,
                    items: sim.responses.len() as i32,
                });
                started = finished + Duration::minutes(rng.random_range(5..120));
                if sim.decision.passed {
                    let bank = assessment::bank();
                    for r in sim.responses.iter().filter(|r| r.1) {
                        if let Some(t) = bank.template(&r.0.template_id) {
                            verified.extend(
                                t.skills
                                    .iter()
                                    .copied()
                                    .filter(|s| matching::lookup(s).is_some()),
                            );
                        }
                    }
                    assigned = Some((
                        try_grade,
                        sim.decision.theta,
                        sim.decision.se,
                        session_id,
                        finished,
                    ));
                    if sim.decision.confident_above_next
                        && kind == "placement"
                        && rng.random_bool(0.5)
                    {
                        if let Some(up) = try_grade.next() {
                            try_grade = up;
                            kind = "placement_upgrade";
                            continue;
                        }
                    }
                    break;
                }
                if assigned.is_some() || kind != "placement" {
                    break;
                }
                match try_grade.prev() {
                    Some(lower) => {
                        try_grade = lower;
                        kind = "placement_step_down";
                    }
                    None => break,
                }
            }
            for skill in &verified {
                db::candidates::insert_candidate_skill(&mut *tx, acc.id, skill, "test").await?;
            }
            let mut remarks = false;
            if let Some(p) = seed_proctoring(
                &mut tx,
                &mut proctoring_rng,
                acc.id,
                &stored,
                now,
                retention_days,
            )
            .await?
            {
                if p.camera == Camera::Declined {
                    without_video += 1;
                } else {
                    with_video += 1;
                }
                with_remarks += usize::from(p.remarks);
                remarks = p.remarks;
            }
            if let Some((_, _, _, session, finished)) = assigned
                && n != 0
            {
                reviewables.push(moderation::Reviewable {
                    candidate: acc.id,
                    session,
                    finished,
                    remarks,
                });
            }
            everyone.push(acc.id);
            if let Some((g, theta, se, session_id, at)) = assigned {
                let reason = if g > claimed { "upgrade" } else { "placement" };
                db::grades::insert_grade_assignment(
                    &mut *tx,
                    &db::grades::NewGradeAssignment {
                        id: Uuid::now_v7(),
                        candidate_id: acc.id,
                        specialization: spec.slug(),
                        grade: g.slug(),
                        theta,
                        se,
                        session_id: Some(session_id),
                        reason,
                        assigned_at: at,
                    },
                )
                .await?;
                db::grades::set_candidate_category(
                    &mut *tx,
                    spec.slug(),
                    g.slug(),
                    theta,
                    at,
                    acc.id,
                )
                .await?;
            }
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
                db::seed::seed_fsp_link(&mut *tx, acc.id, &fsp_id, &raw).await?;
                crate::skills::replace_fsp_skills(&mut tx, acc.id, Some(&raw)).await?;
                fsp_linked += 1;
            }
            if matches!(assigned, Some((Grade::Junior, ..))) && n != 0 {
                juniors.push(Junior {
                    id: acc.id,
                    spec,
                    created,
                    surname: last_pair,
                });
            }
            n += 1;
        }
    }
    let minors: Vec<Uuid> = Specialization::ALL
        .iter()
        .filter_map(|spec| juniors.iter().find(|j| j.spec == *spec))
        .map(|j| j.id)
        .collect();
    reviewables.retain(|r| !minors.contains(&r.candidate));
    let mut review = moderation::ModerationReport::default();
    moderation::seed_moderator(&mut tx, crypto, &hash, now).await?;
    let reviewed = moderation::seed_reviews(
        &mut tx,
        crypto,
        &mut moderation_rng,
        &reviewables,
        &mut review,
    )
    .await?;
    let reporters: Vec<Uuid> = everyone
        .iter()
        .copied()
        .skip(1)
        .filter(|id| !reviewed.contains(id) && !minors.contains(id))
        .step_by(7)
        .take(3)
        .collect();
    moderation::seed_trust(
        &mut tx,
        &mut moderation_rng,
        &companies,
        &reporters,
        now,
        &mut review,
    )
    .await?;
    let (minors_confirmed, minors_pending) =
        seed_minors(&mut tx, crypto, &mut minors_rng, &juniors, now).await?;
    tx.commit().await?;
    Ok(Some(SeedReport {
        companies: companies.len(),
        candidates: n,
        fsp_linked,
        with_video,
        without_video,
        with_remarks,
        minors_confirmed,
        minors_pending,
        review_flagged: review.flagged_with_events,
        review_unproctored: review.flagged_without_proctoring,
        review_remarks: review.remarks_only,
        flagged_vacancies: review.flagged_vacancies,
        complaints: review.complaints,
    }))
}
