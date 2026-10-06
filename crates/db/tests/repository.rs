use chrono::{Duration, Utc};
use db::types::{AssessmentStatus, InvitationStatus, UserRole};
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
    db::candidates::insert_candidate(pool, id, &format!("K-{}", &email[..6]))
        .await
        .unwrap();
    id
}

async fn company(pool: &PgPool) -> Uuid {
    let owner = user(pool, UserRole::Employer, "hr@example.ru").await;
    let id = Uuid::now_v7();
    db::companies::insert_company(pool, id, owner, "Рога и копыта")
        .await
        .unwrap();
    id
}

fn invitation(company_id: Uuid, candidate_id: Uuid) -> db::invitations::NewInvitation<'static> {
    db::invitations::NewInvitation {
        id: Uuid::now_v7(),
        company_id,
        candidate_id,
        need_id: None,
        vacancy_id: None,
        title: "Backend-разработчик",
        message: "Здравствуйте!",
        salary_min: 200_000,
        salary_max: 300_000,
        work_format: Some("remote"),
        contact: "@hr",
        idempotency_key: None,
        expires_at: Utc::now() + Duration::days(14),
    }
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn user_is_found_by_blind_index(pool: PgPool) {
    let id = Uuid::now_v7();
    let bidx = [7u8; 32];
    db::users::insert_user(&pool, id, UserRole::Employer, b"enc", &bidx, "phc")
        .await
        .unwrap();

    let found = db::users::user_by_email_bidx(&pool, &bidx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.role, UserRole::Employer);
    assert_eq!(found.email_enc, b"enc");
    assert_eq!(found.password_hash, "phc");
    assert!(found.email_verified_at.is_none() && found.deleted_at.is_none());

    assert!(
        db::users::user_by_email_bidx(&pool, &[8u8; 32])
            .await
            .unwrap()
            .is_none()
    );
    let duplicate =
        db::users::insert_user(&pool, Uuid::now_v7(), UserRole::Candidate, b"x", &bidx, "y")
            .await
            .unwrap_err();
    assert!(db::is_unique_violation(&duplicate));
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn one_active_invitation_per_company_and_candidate(pool: PgPool) {
    let company_id = company(&pool).await;
    let candidate_id = candidate(&pool, "ivan@example.ru").await;

    let first = invitation(company_id, candidate_id);
    db::invitations::insert_invitation(&pool, &first)
        .await
        .unwrap();
    let second = db::invitations::insert_invitation(&pool, &invitation(company_id, candidate_id))
        .await
        .unwrap_err();
    assert!(db::is_unique_violation(&second));

    assert_eq!(
        db::invitations::withdraw_invitation(&pool, first.id, company_id)
            .await
            .unwrap(),
        1
    );
    db::invitations::insert_invitation(&pool, &invitation(company_id, candidate_id))
        .await
        .unwrap();
    let statuses: Vec<InvitationStatus> = db::invitations::employer_invitations(&pool, company_id)
        .await
        .unwrap()
        .into_iter()
        .map(|i| i.status)
        .collect();
    assert_eq!(statuses.len(), 2);
    assert!(statuses.contains(&InvitationStatus::Sent));
    assert!(statuses.contains(&InvitationStatus::Withdrawn));
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn shortlist_keeps_pins_across_generations(pool: PgPool) {
    let company_id = company(&pool).await;
    let a = candidate(&pool, "anna@example.ru").await;
    let b = candidate(&pool, "boris@example.ru").await;
    let need_id = Uuid::now_v7();
    let grades = ["middle".to_owned()];
    db::needs::insert_need(
        &pool,
        need_id,
        company_id,
        &db::needs::NeedFields {
            title: "Backend",
            description: "",
            specialization: "backend",
            grades: &grades,
            allow_adjacent_grades: false,
            must_skills: &[],
            nice_skills: &[],
            salary_min: 200_000,
            salary_max: 300_000,
            work_formats: &[],
            city: None,
            industries: &[],
        },
    )
    .await
    .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let first = db::needs::start_shortlist_generation(&mut *tx, &json!({}), need_id)
        .await
        .unwrap();
    assert!(first.previous.is_none());
    for (rank, id) in [a, b].into_iter().enumerate() {
        db::needs::upsert_shortlist_item(&mut *tx, need_id, id, 0.9, rank as i32 + 1, &json!({}))
            .await
            .unwrap();
    }
    db::needs::mark_shortlist_stale(&mut *tx, need_id, &[a, b])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(
        db::needs::patch_shortlist_item(&pool, Some(true), None, company_id, need_id, b)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        db::needs::patch_shortlist_item(&pool, Some(true), None, Uuid::now_v7(), need_id, b)
            .await
            .unwrap(),
        0,
        "another company cannot touch the shortlist"
    );

    let mut tx = pool.begin().await.unwrap();
    let second =
        db::needs::start_shortlist_generation(&mut *tx, &json!({"fsp_only": true}), need_id)
            .await
            .unwrap();
    assert_eq!(second.previous, Some(first.generated_at));
    db::needs::upsert_shortlist_item(&mut *tx, need_id, a, 0.95, 1, &json!({}))
        .await
        .unwrap();
    db::needs::mark_shortlist_stale(&mut *tx, need_id, &[a])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let items = db::needs::shortlist_items(&pool, need_id).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!((items[0].candidate_id, items[0].in_current), (a, true));
    assert_eq!(
        (items[1].candidate_id, items[1].in_current, items[1].pinned),
        (b, false, true)
    );

    let need = db::needs::company_need(&pool, company_id, need_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(need.shortlisted, 1);
    assert_eq!(need.filters, json!({"fsp_only": true}));
    assert_eq!(need.shortlist_generated_at, Some(second.generated_at));
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn assessment_session_round_trip(pool: PgPool) {
    let candidate_id = candidate(&pool, "maria@example.ru").await;
    let id = Uuid::now_v7();
    let now = Utc::now();
    db::assessment::insert_session(
        &pool,
        &db::assessment::NewSession {
            id,
            candidate_id,
            specialization: "backend",
            claimed_grade: "middle",
            attempt_kind: "placement",
            state: &json!({"step": 0}),
            current_item: &json!({"template_id": "t1"}),
            deadline: now + Duration::minutes(3),
            shown_at: now,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        db::assessment::active_session(&pool, candidate_id)
            .await
            .unwrap(),
        Some(id)
    );

    let mut tx = pool.begin().await.unwrap();
    let row = db::assessment::session_for_update(&mut *tx, id, candidate_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.status, AssessmentStatus::InProgress);
    assert_eq!(row.current_item, Some(json!({"template_id": "t1"})));
    db::assessment::insert_response(
        &mut *tx,
        &db::assessment::NewResponse {
            session_id: id,
            seq: 0,
            template_id: "t1",
            level: 2,
            seed: 42,
            answer: Some(&json!({"choice": 1})),
            correct: true,
            timed_out: false,
            elapsed_ms: 12_000,
            theta_after: 0.4,
            se_after: 0.8,
            answered_at: now,
        },
    )
    .await
    .unwrap();
    db::assessment::finish_session(
        &mut *tx,
        id,
        &db::assessment::FinishedSession {
            status: AssessmentStatus::Passed,
            state: &json!({"step": 1}),
            theta: 0.4,
            se: 0.3,
            measured_grade: Some("middle"),
            confident_above_next: false,
            person_fit_lz: 0.1,
            aberrant: false,
            items_count: 1,
            finished_at: now,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let s = db::assessment::session_state(&pool, id, candidate_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(s.status, AssessmentStatus::Passed);
    assert_eq!(s.state, json!({"step": 1}));
    assert!(s.current_item.is_none() && s.current_item_deadline.is_none());
    assert_eq!(
        (s.theta, s.measured_grade.as_deref()),
        (Some(0.4), Some("middle"))
    );
    assert_eq!(s.items_count, 1);
    assert!(!s.assigned);
    assert!(
        db::assessment::session_state(&pool, id, Uuid::now_v7())
            .await
            .unwrap()
            .is_none(),
        "a session is visible only to its candidate"
    );
    assert!(
        db::assessment::active_session(&pool, candidate_id)
            .await
            .unwrap()
            .is_none()
    );
    let history = db::assessment::candidate_sessions(&pool, candidate_id)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, id);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn pool_rows_without_fsp_link(pool: PgPool) {
    let company_id = company(&pool).await;
    let id = candidate(&pool, "oleg@example.ru").await;
    db::grades::set_candidate_category(&pool, "backend", "middle", 0.3, Utc::now(), id)
        .await
        .unwrap();
    let publication = db::consents::current_document(&pool, db::consents::PUBLICATION)
        .await
        .unwrap()
        .unwrap();
    db::consents::insert_user_consent(&pool, Uuid::now_v7(), id, publication.id, None, None)
        .await
        .unwrap();
    db::candidates::insert_candidate_skill(&pool, id, "rust", "declared")
        .await
        .unwrap();
    db::candidates::insert_candidate_skill(&pool, id, "rust", "test")
        .await
        .unwrap();

    let rows = db::pool::searchable_pool(&pool, "backend").await.unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].achievements.is_none() && !rows[0].minor);
    assert_eq!(rows[0].skills, ["rust", "rust"]);
    assert_eq!(rows[0].skill_sources, ["declared", "test"]);

    assert!(
        db::pool::text_ranks(&pool, "backend", "rust | tokio")
            .await
            .unwrap()
            .is_empty(),
        "an empty «about» matches nothing"
    );
    db::candidates::update_candidate_profile(
        &pool,
        id,
        &db::candidates::ProfileUpdate {
            full_name_enc: None,
            contacts_enc: None,
            city: None,
            relocation: false,
            work_formats: &[],
            salary_expectation_min: None,
            about: "Пишу сервисы на Rust и Tokio",
            experience_years: None,
            roles: &[],
            soft_skills: &[],
            industries: &[],
            experience: &json!([]),
        },
    )
    .await
    .unwrap();
    let ranks = db::pool::text_ranks(&pool, "backend", "rust | tokio")
        .await
        .unwrap();
    assert_eq!(ranks.len(), 1);
    assert_eq!(ranks[0].user_id, id);
    assert!(ranks[0].rank > 0.0);
    assert!(
        db::pool::text_ranks(&pool, "frontend", "rust | tokio")
            .await
            .unwrap()
            .is_empty()
    );

    let cards = db::pool::candidate_cards(&pool, &[id]).await.unwrap();
    assert_eq!(cards.len(), 1);
    assert!(cards[0].achievements.is_none() && cards[0].task_rating.is_none());
    assert_eq!(cards[0].tasks_done, 0);

    let relations = db::pool::company_relations(&pool, company_id, &[id])
        .await
        .unwrap();
    assert!(relations[0].invitation_status.is_none() && !relations[0].applied);
}
