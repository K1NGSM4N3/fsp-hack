//! Demo notifications: a few read and unread items for `candidate@demo.fsp` and
//! `employer@demo.fsp`, each about a real invitation, application or test seeded here, so every
//! link in the bell leads somewhere. No randomness, so the rest of the seed stays the same; the
//! demo company gets no acceptance, so its trust level and quota do not move.

use catalog::{Category, WorkFormat};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgConnection;
use uuid::{NoContext, Timestamp, Uuid};

use super::{corpus, demo::DEMO_COMPANY};
use crate::{
    crypto::Crypto,
    dto::DeclineReason,
    error::ApiResult,
    notify::{About, ApplicantNote, AssessmentNote, DeclinedNote, Event, InvitationNote, To},
    routes::invitations::TTL_DAYS,
};

const OFFERS: [(&str, &str, (i64, i64)); 2] = [
    (
        "Backend-разработчик в команду игровых серверов",
        "Здравствуйте! Делаем серверную часть онлайн-игр: матчмейкинг, платежи, античит. Посмотрели ваш профиль — по стеку и результату теста вы нам подходите. Расскажем подробнее на созвоне.",
        (250_000, 320_000),
    ),
    (
        "Go-разработчик во внутреннюю платформу",
        "Добрый день! Нужен человек во внутреннюю платформу: API для команд, очереди, наблюдаемость.",
        (220_000, 290_000),
    ),
];

struct Item {
    to: To,
    about: About,
    event: Event,
    created_at: DateTime<Utc>,
    read_at: Option<DateTime<Utc>>,
}

async fn user_id(tx: &mut PgConnection, crypto: &Crypto, email: &str) -> ApiResult<Option<Uuid>> {
    Ok(
        db::users::user_by_email_bidx(&mut *tx, &crypto.blind_index(email))
            .await?
            .map(|u| u.id),
    )
}

/// Adults among `cand1…cand10@demo.fsp`, in email order.
async fn adult_candidates(
    tx: &mut PgConnection,
    crypto: &Crypto,
    n: usize,
) -> ApiResult<Vec<Uuid>> {
    let mut out = Vec::with_capacity(n);
    for i in 1..=10 {
        if out.len() == n {
            break;
        }
        let Some(id) = user_id(tx, crypto, &format!("cand{i}@demo.fsp")).await? else {
            continue;
        };
        if db::guardians::candidate_age(&mut *tx, id)
            .await?
            .is_some_and(|a| !a.minor)
        {
            out.push(id);
        }
    }
    Ok(out)
}

async fn invite(
    tx: &mut PgConnection,
    company: Uuid,
    candidate: Uuid,
    offer: (&str, &str, (i64, i64)),
    contact: &str,
    now: DateTime<Utc>,
) -> ApiResult<Uuid> {
    let id = Uuid::now_v7();
    db::invitations::insert_invitation(
        &mut *tx,
        &db::invitations::NewInvitation {
            id,
            company_id: company,
            candidate_id: candidate,
            need_id: None,
            vacancy_id: None,
            title: offer.0,
            message: offer.1,
            salary_min: offer.2.0,
            salary_max: offer.2.1,
            work_format: Some(WorkFormat::Remote.slug()),
            contact,
            idempotency_key: None,
            expires_at: now + Duration::days(TTL_DAYS),
        },
    )
    .await?;
    db::invitations::insert_invitation_event(
        &mut *tx,
        Uuid::now_v7(),
        id,
        db::types::InvitationStatus::Sent,
        None,
    )
    .await?;
    Ok(id)
}

fn note(
    id: Uuid,
    company: usize,
    offer: (&str, &str, (i64, i64)),
    now: DateTime<Utc>,
) -> InvitationNote {
    InvitationNote {
        invitation_id: id,
        company_name: corpus::COMPANIES[company].name.to_owned(),
        title: offer.0.to_owned(),
        salary_min: offer.2.0,
        salary_max: offer.2.1,
        work_format: Some(WorkFormat::Remote),
        expires_at: now + Duration::days(TTL_DAYS),
    }
}

async fn candidate_items(
    tx: &mut PgConnection,
    crypto: &Crypto,
    companies: &[Uuid],
    now: DateTime<Utc>,
) -> ApiResult<Vec<Item>> {
    let Some(candidate) = user_id(tx, crypto, "candidate@demo.fsp").await? else {
        return Ok(Vec::new());
    };
    let mut items = Vec::new();
    let assigned = db::grades::grade_history(&mut *tx, candidate).await?;
    if let Some(g) = assigned.first()
        && let Some(session_id) = g.session_id
    {
        items.push(Item {
            to: To::Candidate(candidate),
            about: About::default(),
            event: Event::AssessmentResult(AssessmentNote {
                session_id,
                category: Category::new(g.specialization.parse()?, g.grade.parse()?),
                passed: true,
            }),
            created_at: g.assigned_at,
            read_at: Some(g.assigned_at + Duration::minutes(5)),
        });
    }

    let (open, gone) = (1, 2);
    let withdrawn = invite(
        tx,
        companies[gone],
        candidate,
        OFFERS[1],
        corpus::COMPANIES[gone].telegram,
        now,
    )
    .await?;
    db::invitations::withdraw_invitation(&mut *tx, withdrawn, companies[gone]).await?;
    db::invitations::insert_invitation_event(
        &mut *tx,
        Uuid::now_v7(),
        withdrawn,
        db::types::InvitationStatus::Withdrawn,
        None,
    )
    .await?;
    items.push(Item {
        to: To::Candidate(candidate),
        about: About {
            candidate_id: None,
            company_id: Some(companies[gone]),
        },
        event: Event::InvitationWithdrawn(note(withdrawn, gone, OFFERS[1], now)),
        created_at: now - Duration::hours(26),
        read_at: Some(now - Duration::hours(20)),
    });

    let received = invite(
        tx,
        companies[open],
        candidate,
        OFFERS[0],
        corpus::COMPANIES[open].telegram,
        now,
    )
    .await?;
    items.push(Item {
        to: To::Candidate(candidate),
        about: About {
            candidate_id: None,
            company_id: Some(companies[open]),
        },
        event: Event::InvitationReceived(note(received, open, OFFERS[0], now)),
        created_at: now - Duration::hours(2),
        read_at: None,
    });
    Ok(items)
}

async fn employer_items(
    tx: &mut PgConnection,
    crypto: &Crypto,
    companies: &[Uuid],
    now: DateTime<Utc>,
) -> ApiResult<Vec<Item>> {
    let company = companies[DEMO_COMPANY];
    let people = adult_candidates(tx, crypto, 3).await?;
    let Some(vacancy) = db::vacancies::company_vacancies(&mut *tx, company)
        .await?
        .into_iter()
        .find(|v| v.status == db::types::VacancyStatus::Published)
    else {
        return Ok(Vec::new());
    };
    let mut items = Vec::new();
    let letters = [
        (
            "Здравствуйте! Пишу сервисы на Go третий год, с платёжными API работал в двух проектах. Буду рад обсудить.",
            Duration::minutes(40),
            None,
        ),
        (
            "Добрый день! Интересна ваша вакансия: процессинг и outbox — ровно то, чем занимаюсь сейчас.",
            Duration::hours(27),
            Some(Duration::hours(25)),
        ),
    ];
    for (&candidate, (letter, ago, read)) in people.iter().zip(letters) {
        let id = Uuid::now_v7();
        db::vacancies::insert_application(&mut *tx, id, vacancy.id, candidate, letter).await?;
        let code = db::integrations::candidate_brief(&mut *tx, candidate)
            .await?
            .map(|b| b.public_code)
            .unwrap_or_default();
        items.push(Item {
            to: To::Company(company),
            about: About {
                candidate_id: Some(candidate),
                company_id: None,
            },
            event: Event::ApplicationReceived(ApplicantNote {
                application_id: id,
                vacancy_id: vacancy.id,
                vacancy_title: vacancy.title.clone(),
                candidate_id: candidate,
                candidate_code: code,
            }),
            created_at: now - ago,
            read_at: read.map(|r| now - r),
        });
    }

    if let Some(&candidate) = people.get(2) {
        let offer = (
            "Backend-разработчик в команду платежей",
            "Здравствуйте! Ищем разработчика в процессинг: Go, PostgreSQL, Kafka.",
            (200_000, 240_000),
        );
        let id = invite(
            tx,
            company,
            candidate,
            offer,
            corpus::COMPANIES[DEMO_COMPANY].telegram,
            now,
        )
        .await?;
        let comment = "Спасибо, но рассматриваю предложения от 300 тысяч.";
        db::invitations::respond_invitation(
            &mut *tx,
            db::types::InvitationStatus::Declined,
            Some(DeclineReason::Salary.slug()),
            Some(comment),
            id,
            candidate,
        )
        .await?;
        db::invitations::insert_invitation_event(
            &mut *tx,
            Uuid::now_v7(),
            id,
            db::types::InvitationStatus::Declined,
            Some(candidate),
        )
        .await?;
        let code = db::integrations::candidate_brief(&mut *tx, candidate)
            .await?
            .map(|b| b.public_code)
            .unwrap_or_default();
        items.push(Item {
            to: To::Company(company),
            about: About {
                candidate_id: Some(candidate),
                company_id: None,
            },
            event: Event::InvitationDeclined(DeclinedNote {
                invitation_id: id,
                candidate_code: code,
                title: offer.0.to_owned(),
                reason: Some(DeclineReason::Salary),
                comment: Some(comment.to_owned()),
            }),
            created_at: now - Duration::hours(3),
            read_at: Some(now - Duration::hours(2)),
        });
    }
    Ok(items)
}

/// Returns how many items were written. Called last in the seed transaction.
pub async fn seed_notifications(
    tx: &mut PgConnection,
    crypto: &Crypto,
    companies: &[Uuid],
    now: DateTime<Utc>,
) -> ApiResult<usize> {
    let mut items = candidate_items(tx, crypto, companies, now).await?;
    items.extend(employer_items(tx, crypto, companies, now).await?);
    items.sort_by_key(|i| i.created_at);
    for item in &items {
        let payload = serde_json::to_string(&item.event)?;
        let kind = item.event.kind();
        let (user_id, company_id, candidate_id) = match item.to {
            To::Candidate(id) => (Some(id), item.about.company_id, Some(id)),
            To::Company(id) => (None, Some(id), item.about.candidate_id),
        };
        let at = item.created_at;
        let ts = Timestamp::from_unix(
            NoContext,
            u64::try_from(at.timestamp()).unwrap_or(0),
            at.timestamp_subsec_nanos(),
        );
        db::notifications::insert_seeded(
            &mut *tx,
            &db::notifications::NewNotification {
                id: Uuid::new_v7(ts),
                user_id,
                kind: kind.slug(),
                kind_bit: kind.bit(),
                payload: &payload,
                subject_id: item.event.subject(),
                candidate_id,
                company_id,
                dedup_key: None,
                email: false,
                hold_secs: 0.0,
            },
            item.created_at,
            item.read_at,
        )
        .await?;
    }
    Ok(items.len())
}
