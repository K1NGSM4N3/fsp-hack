use chrono::{DateTime, Duration, Utc};
use db::types::{AssessmentStatus, IntegrityEvent, UserRole};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn candidate(pool: &PgPool, code: &str) -> Uuid {
    let id = Uuid::now_v7();
    db::users::insert_user(
        pool,
        id,
        UserRole::Candidate,
        code.as_bytes(),
        code.as_bytes(),
        "h",
    )
    .await
    .unwrap();
    db::candidates::insert_candidate(pool, id, code)
        .await
        .unwrap();
    id
}

async fn session(pool: &PgPool, candidate_id: Uuid, started: DateTime<Utc>) -> Uuid {
    db::assessment::expire_abandoned_sessions(pool, candidate_id)
        .await
        .unwrap();
    let id = Uuid::now_v7();
    db::assessment::insert_session(
        pool,
        &db::assessment::NewSession {
            id,
            candidate_id,
            specialization: "backend",
            claimed_grade: "middle",
            attempt_kind: "placement",
            state: &json!({}),
            current_item: &json!({}),
            deadline: started + Duration::minutes(3),
            shown_at: started,
        },
    )
    .await
    .unwrap();
    id
}

async fn proctoring_consent(pool: &PgPool, user_id: Uuid) -> Uuid {
    let doc = db::consents::current_document(pool, "proctoring_video")
        .await
        .unwrap()
        .expect("seeded by the migration");
    assert!(
        doc.required_for.is_empty(),
        "optional, not asked at registration"
    );
    let grant = Uuid::now_v7();
    db::consents::insert_user_consent(pool, grant, user_id, doc.id, None, None)
        .await
        .unwrap();
    assert_eq!(
        db::consents::active_grant(pool, user_id, doc.id)
            .await
            .unwrap(),
        Some(grant)
    );
    grant
}

fn event(kind: IntegrityEvent, at: DateTime<Utc>) -> db::proctoring::Event {
    db::proctoring::Event {
        kind,
        occurred_at: at,
        item_seq: Some(2),
        duration_ms: None,
    }
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn chunk_metadata_events_and_badge(pool: PgPool) {
    let now = Utc::now();
    let cand = candidate(&pool, "K-PR0001").await;
    let sid = session(&pool, cand, now).await;
    let grant = proctoring_consent(&pool, cand).await;
    db::proctoring::insert_proctoring(&pool, sid, cand, true, Some(grant), now)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let t = db::proctoring::upload_target(&mut *tx, sid, cand)
        .await
        .unwrap()
        .unwrap();
    assert!(t.camera && t.consent_active && !t.purged);
    assert_eq!(t.status, AssessmentStatus::InProgress);
    assert_eq!((t.chunks, t.bytes, t.events), (0, 0, 0));
    db::proctoring::insert_chunk(&mut *tx, sid, 0, 1200)
        .await
        .unwrap();
    db::proctoring::insert_chunk(&mut *tx, sid, 1, 800)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let dup = db::proctoring::insert_chunk(&pool, sid, 1, 800)
        .await
        .unwrap_err();
    assert!(db::is_unique_violation(&dup));
    assert!(
        db::proctoring::upload_target(&pool, sid, Uuid::now_v7())
            .await
            .unwrap()
            .is_none(),
        "another candidate cannot upload into this session"
    );

    for kind in [
        IntegrityEvent::TabHidden,
        IntegrityEvent::TabHidden,
        IntegrityEvent::Paste,
    ] {
        db::proctoring::insert_event(&pool, sid, &event(kind, now))
            .await
            .unwrap();
    }
    let chunks = db::proctoring::chunks(&pool, sid).await.unwrap();
    assert_eq!(
        chunks.iter().map(|c| (c.seq, c.size)).collect::<Vec<_>>(),
        [(0, 1200), (1, 800)]
    );
    let s = db::proctoring::summary(&pool, sid, Some(cand))
        .await
        .unwrap()
        .unwrap();
    assert!(s.camera && s.purged_at.is_none());
    assert_eq!((s.chunks, s.bytes), (2, 2000));
    assert_eq!(
        (s.tab_hidden, s.paste, s.window_blur, s.camera_lost),
        (2, 1, 0, 0)
    );
    assert!(
        db::proctoring::summary(&pool, sid, Some(Uuid::now_v7()))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        db::proctoring::summary(&pool, sid, None)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(db::proctoring::events(&pool, sid).await.unwrap().len(), 3);

    let card = |pool: PgPool| async move {
        db::pool::candidate_cards(&pool, Uuid::now_v7(), &[cand])
            .await
            .unwrap()
            .pop()
            .unwrap()
    };
    assert!(card(pool.clone()).await.proctoring().is_none());
    db::grades::insert_grade_assignment(
        &pool,
        &db::grades::NewGradeAssignment {
            id: Uuid::now_v7(),
            candidate_id: cand,
            specialization: "backend",
            grade: "middle",
            theta: 0.4,
            se: 0.3,
            session_id: Some(sid),
            reason: "placement",
            assigned_at: now,
        },
    )
    .await
    .unwrap();
    let badge = card(pool.clone()).await.proctoring().unwrap();
    assert!(badge.camera);
    assert_eq!((badge.chunks, badge.tab_hidden, badge.paste), (2, 2, 1));

    db::consents::revoke_user_consent(&pool, grant, cand)
        .await
        .unwrap();
    let t = db::proctoring::upload_target(&pool, sid, cand)
        .await
        .unwrap()
        .unwrap();
    assert!(!t.consent_active, "revoked consent stops uploads");
    assert_eq!(t.events, 3);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn retention_purges_only_old_recordings(pool: PgPool) {
    let now = Utc::now();
    let cutoff = now - Duration::days(90);
    let cand = candidate(&pool, "K-PR0002").await;
    let grant = proctoring_consent(&pool, cand).await;

    let old = session(&pool, cand, now - Duration::days(91)).await;
    db::proctoring::insert_proctoring(
        &pool,
        old,
        cand,
        true,
        Some(grant),
        now - Duration::days(91),
    )
    .await
    .unwrap();
    db::proctoring::insert_chunk(&pool, old, 0, 500)
        .await
        .unwrap();
    db::proctoring::insert_event(&pool, old, &event(IntegrityEvent::WindowBlur, now))
        .await
        .unwrap();

    let declined = session(&pool, cand, now - Duration::days(120)).await;
    db::proctoring::insert_proctoring(
        &pool,
        declined,
        cand,
        false,
        None,
        now - Duration::days(120),
    )
    .await
    .unwrap();

    let fresh = session(&pool, cand, now - Duration::days(3)).await;
    db::proctoring::insert_proctoring(
        &pool,
        fresh,
        cand,
        true,
        Some(grant),
        now - Duration::days(3),
    )
    .await
    .unwrap();
    db::proctoring::insert_chunk(&pool, fresh, 0, 700)
        .await
        .unwrap();

    let expired = db::proctoring::expired_recordings(&pool, cutoff, 100)
        .await
        .unwrap();
    assert_eq!(
        expired.iter().map(|r| r.session_id).collect::<Vec<_>>(),
        [old],
        "only camera recordings past retention; declined sessions have no video"
    );
    assert_eq!(expired[0].candidate_id, cand);

    assert_eq!(db::proctoring::mark_purged(&pool, &[old]).await.unwrap(), 1);
    assert_eq!(db::proctoring::mark_purged(&pool, &[old]).await.unwrap(), 0);
    let s = db::proctoring::summary(&pool, old, Some(cand))
        .await
        .unwrap()
        .unwrap();
    assert!(s.purged_at.is_some());
    assert_eq!(
        (s.chunks, s.window_blur),
        (1, 1),
        "the summary outlives the video and still reads as recorded"
    );
    assert!(
        db::proctoring::expired_recordings(&pool, cutoff, 100)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(db::proctoring::chunks(&pool, fresh).await.unwrap().len(), 1);

    let live = db::proctoring::candidate_recordings(&pool, cand)
        .await
        .unwrap();
    assert_eq!(
        live.iter().map(|r| r.session_id).collect::<Vec<_>>(),
        [fresh]
    );

    let stranger = Uuid::now_v7();
    assert_eq!(
        db::proctoring::known_candidates(&pool, &[cand, stranger])
            .await
            .unwrap(),
        [cand]
    );
    db::me::delete_candidate(&pool, cand).await.unwrap();
    assert!(
        db::proctoring::known_candidates(&pool, &[cand])
            .await
            .unwrap()
            .is_empty(),
        "account deletion cascades to proctoring metadata"
    );
}
