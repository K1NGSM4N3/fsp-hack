use chrono::{Duration, Utc};
use db::types::{InvitationStatus, UserRole};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn user(pool: &PgPool, role: UserRole, email: &str) -> Uuid {
    let id = Uuid::now_v7();
    db::users::insert_user(pool, id, role, email.as_bytes(), email.as_bytes(), "hash")
        .await
        .unwrap();
    id
}

async fn candidate(pool: &PgPool, email: &str) -> Uuid {
    let id = user(pool, UserRole::Candidate, email).await;
    db::candidates::insert_candidate(pool, id, &format!("K-{}", id.simple()))
        .await
        .unwrap();
    id
}

async fn invite(pool: &PgPool, company: Uuid, candidate: Uuid, ttl_days: i64) -> Uuid {
    let id = Uuid::now_v7();
    db::invitations::insert_invitation(
        pool,
        &db::invitations::NewInvitation {
            id,
            company_id: company,
            candidate_id: candidate,
            need_id: None,
            vacancy_id: None,
            title: "Backend-разработчик",
            message: "Здравствуйте!",
            salary_min: 200_000,
            salary_max: 300_000,
            work_format: None,
            contact: "@hr",
            idempotency_key: None,
            expires_at: Utc::now() + Duration::days(ttl_days),
        },
    )
    .await
    .unwrap();
    id
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn company_signals_count_behaviour_flags_and_complaints(pool: PgPool) {
    let owner = user(&pool, UserRole::Employer, "hr@corp.ru").await;
    let company = Uuid::now_v7();
    db::companies::insert_company(&pool, company, owner, "Корп")
        .await
        .unwrap();
    let other_owner = user(&pool, UserRole::Employer, "hr@other.ru").await;
    let other = Uuid::now_v7();
    db::companies::insert_company(&pool, other, other_owner, "Другая")
        .await
        .unwrap();

    let mut people = Vec::new();
    for i in 0..6 {
        people.push(candidate(&pool, &format!("person{i}@example.ru")).await);
    }
    let respond = |status, reason: Option<&'static str>, id, who| {
        let pool = pool.clone();
        async move {
            db::invitations::respond_invitation(&pool, status, reason, None, id, who)
                .await
                .unwrap()
        }
    };
    let a = invite(&pool, company, people[0], 14).await;
    respond(InvitationStatus::Accepted, None, a, people[0]).await;
    let b = invite(&pool, company, people[1], 14).await;
    respond(InvitationStatus::Declined, Some("salary"), b, people[1]).await;
    let c = invite(&pool, company, people[2], 14).await;
    respond(InvitationStatus::Declined, Some("stack"), c, people[2]).await;
    let d = invite(&pool, company, people[3], 14).await;
    db::invitations::withdraw_invitation(&pool, d, company)
        .await
        .unwrap();
    invite(&pool, company, people[4], -1).await;
    db::invitations::expire_stale_invitations(&pool)
        .await
        .unwrap();
    let flagged = invite(&pool, company, people[5], 14).await;
    db::trust::set_invitation_flags(
        &pool,
        flagged,
        &json!([{ "kind": "salary_wide_range", "min": 100000, "max": 400000 }]),
    )
    .await
    .unwrap();

    let vacancy = Uuid::now_v7();
    db::vacancies::insert_vacancy(
        &pool,
        vacancy,
        company,
        db::types::VacancyStatus::Published,
        Some(Utc::now()),
        &db::vacancies::VacancyFields {
            title: "Go",
            description: "",
            specialization: "backend",
            grade: "middle",
            skills: &[],
            salary_min: 100_000,
            salary_max: 900_000,
            work_format: "remote",
            city: None,
        },
    )
    .await
    .unwrap();
    db::trust::set_vacancy_flags(
        &pool,
        vacancy,
        &json!([{ "kind": "salary_wide_range", "min": 100000, "max": 900000 },
                { "kind": "copy_of_other", "vacancy_id": "x", "company": "Другая", "title": "Go", "similarity": 0.9 }]),
    )
    .await
    .unwrap();

    for (reporter, kind, target) in [
        (people[0], "company", company),
        (people[0], "vacancy", vacancy),
        (people[1], "invitation", b),
        (owner, "company", other),
        (other_owner, "company", company),
    ] {
        db::reports::insert_report(&pool, Uuid::now_v7(), reporter, kind, target, "spam", None)
            .await
            .unwrap();
    }

    let rows = db::trust::company_signals(&pool, None).await.unwrap();
    assert_eq!(rows.len(), 2);
    let s = rows.iter().find(|r| r.company_id == company).unwrap();
    assert_eq!(
        (s.invitations, s.withdrawn, s.closed, s.answered, s.accepted),
        (6, 1, 4, 3, 1)
    );
    assert_eq!((s.declines, s.bad_declines), (2, 1));
    assert_eq!(s.salary_flags, 2, "one invitation and one vacancy");
    assert_eq!((s.own_duplicates, s.copies), (0, 1));
    assert_eq!(
        s.complaints, 2,
        "distinct candidates only; the rival employer's report does not count"
    );
    assert!(s.verified_domain.is_none());

    let o = &db::trust::company_signals(&pool, Some(other))
        .await
        .unwrap()[0];
    assert_eq!((o.complaints, o.invitations, o.copies), (0, 0, 0));

    db::trust::set_vacancy_flags(&pool, vacancy, &json!([]))
        .await
        .unwrap();
    let s = &db::trust::company_signals(&pool, Some(company))
        .await
        .unwrap()[0];
    assert_eq!(
        (s.salary_flags, s.copies),
        (1, 0),
        "cleared flags stop counting"
    );
    assert!(
        db::trust::flagged_vacancies(&pool, 10)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn expectation_band_uses_the_published_pool(pool: PgPool) {
    let band = db::trust::expectation_band(&pool, "backend", "middle")
        .await
        .unwrap();
    assert_eq!(band.samples, 0);
    assert!(band.p90.is_none());

    let publication = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
    for (i, salary) in [100_000, 200_000, 300_000, 400_000, 500_000]
        .into_iter()
        .enumerate()
    {
        let id = candidate(&pool, &format!("market{i}@example.ru")).await;
        db::grades::set_candidate_category(&pool, "backend", "middle", 0.3, Utc::now(), id)
            .await
            .unwrap();
        db::candidates::update_candidate_profile(
            &pool,
            id,
            &db::candidates::ProfileUpdate {
                full_name_enc: None,
                contacts_enc: None,
                city: None,
                relocation: false,
                work_formats: &[],
                salary_expectation_min: Some(salary),
                about: "",
                experience_years: None,
                roles: &[],
                soft_skills: &[],
                industries: &[],
                experience: &json!([]),
            },
        )
        .await
        .unwrap();
        if i > 0 {
            db::consents::insert_user_consent(&pool, Uuid::now_v7(), id, publication, None, None)
                .await
                .unwrap();
        }
    }
    let band = db::trust::expectation_band(&pool, "backend", "middle")
        .await
        .unwrap();
    assert_eq!(
        band.samples, 4,
        "no publication consent — not part of the market"
    );
    assert_eq!(band.median, Some(350_000));
    assert_eq!(band.p90, Some(470_000));

    // A 15-year-old asking a lot: published only once the legal representative consents, and
    // the market band follows the same rule as the category cards.
    let minor = candidate(&pool, "minor@example.ru").await;
    db::grades::set_candidate_category(&pool, "backend", "middle", 0.3, Utc::now(), minor)
        .await
        .unwrap();
    db::candidates::update_candidate_profile(
        &pool,
        minor,
        &db::candidates::ProfileUpdate {
            full_name_enc: None,
            contacts_enc: None,
            city: None,
            relocation: false,
            work_formats: &[],
            salary_expectation_min: Some(900_000),
            about: "",
            experience_years: None,
            roles: &[],
            soft_skills: &[],
            industries: &[],
            experience: &json!([]),
        },
    )
    .await
    .unwrap();
    let born = (Utc::now() - Duration::days(15 * 366)).date_naive();
    db::guardians::set_birth_date(&pool, minor, born)
        .await
        .unwrap();
    db::consents::insert_user_consent(&pool, Uuid::now_v7(), minor, publication, None, None)
        .await
        .unwrap();
    let band = db::trust::expectation_band(&pool, "backend", "middle")
        .await
        .unwrap();
    assert_eq!(band.samples, 4, "under 18 without the representative");
    assert_eq!(band.p90, Some(470_000));
    let cards = db::market::category_overview(&pool).await.unwrap();
    assert_eq!(cards[0].candidates, 4, "the category cards agree");

    let guardian = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000a01);
    db::consents::insert_user_consent(&pool, Uuid::now_v7(), minor, guardian, None, None)
        .await
        .unwrap();
    let band = db::trust::expectation_band(&pool, "backend", "middle")
        .await
        .unwrap();
    assert_eq!(band.samples, 5);
    assert_eq!(band.p90, Some(740_000));
    let cards = db::market::category_overview(&pool).await.unwrap();
    assert_eq!(cards[0].candidates, 5);
}
