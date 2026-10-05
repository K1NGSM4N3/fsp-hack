use chrono::{Duration, Utc};
use db::types::UserRole;
use sqlx::PgPool;
use uuid::Uuid;

async fn company(pool: &PgPool) -> Uuid {
    let owner = Uuid::now_v7();
    db::users::insert_user(pool, owner, UserRole::Employer, b"enc", b"bidx", "hash")
        .await
        .unwrap();
    let id = Uuid::now_v7();
    db::companies::insert_company(pool, id, owner, "Рога и копыта")
        .await
        .unwrap();
    db::integrations::insert_webhook(
        pool,
        id,
        "https://ats.example.ru/hook",
        b"sealed",
        "whsec_ab…",
    )
    .await
    .unwrap();
    id
}

async fn due_event(pool: &PgPool, company_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    db::integrations::insert_event(
        pool,
        &db::integrations::NewEvent {
            id,
            company_id,
            candidate_id: None,
            event_type: "integration.test",
            payload_enc: b"sealed",
            next_attempt_at: Utc::now() - Duration::seconds(1),
            deadline_at: Utc::now() + Duration::hours(24),
        },
    )
    .await
    .unwrap();
    id
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn replicas_never_claim_the_same_event(pool: PgPool) {
    let company = company(&pool).await;
    let mut all = Vec::new();
    for _ in 0..3 {
        all.push(due_event(&pool, company).await);
    }

    let mut first = pool.begin().await.unwrap();
    let a = db::integrations::claim_due_events(&mut *first, 2, 60.0)
        .await
        .unwrap();
    assert_eq!(a.len(), 2);
    assert!(
        a.iter()
            .all(|e| e.url.as_deref() == Some("https://ats.example.ru/hook"))
    );

    let b = db::integrations::claim_due_events(&pool, 10, 60.0)
        .await
        .unwrap();
    assert_eq!(
        b.len(),
        1,
        "rows locked by the open transaction are skipped, not waited on"
    );
    assert!(a.iter().all(|x| x.id != b[0].id));
    first.commit().await.unwrap();

    let mut claimed: Vec<Uuid> = a.iter().chain(&b).map(|e| e.id).collect();
    claimed.sort();
    all.sort();
    assert_eq!(claimed, all);

    let again = db::integrations::claim_due_events(&pool, 10, 60.0)
        .await
        .unwrap();
    assert!(again.is_empty(), "claimed events are leased, not due");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn expired_lease_makes_an_event_due_again(pool: PgPool) {
    let company = company(&pool).await;
    let id = due_event(&pool, company).await;
    let first = db::integrations::claim_due_events(&pool, 10, 0.0)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    let retried = db::integrations::claim_due_events(&pool, 10, 60.0)
        .await
        .unwrap();
    assert_eq!(
        retried.len(),
        1,
        "a replica that died mid-delivery does not lose the event"
    );
    assert_eq!(retried[0].id, id);

    db::integrations::mark_delivered(&pool, id).await.unwrap();
    let counts = db::integrations::event_counts(&pool, company)
        .await
        .unwrap();
    assert_eq!((counts.pending, counts.delivered, counts.failed), (0, 1, 0));
    assert_eq!(
        db::integrations::purge_settled_events(&pool, Utc::now() + Duration::seconds(1))
            .await
            .unwrap(),
        1
    );
}
