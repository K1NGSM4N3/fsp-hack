use chrono::Utc;
use db::types::UserRole;
use sqlx::PgPool;
use uuid::Uuid;

const PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const TERMS_V1: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);
const VIDEO: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000004);
const TERMS_V2: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_0000000000f3);

async fn user(pool: &PgPool, role: UserRole, tag: &str) -> Uuid {
    let id = Uuid::now_v7();
    db::users::insert_user(pool, id, role, tag.as_bytes(), tag.as_bytes(), "h")
        .await
        .unwrap();
    id
}

/// A categorized candidate who registered with every required consent.
async fn published_candidate(pool: &PgPool, code: &str) -> Uuid {
    let id = user(pool, UserRole::Candidate, code).await;
    db::candidates::insert_candidate(pool, id, code)
        .await
        .unwrap();
    db::grades::set_candidate_category(pool, "backend", "middle", 0.4, Utc::now(), id)
        .await
        .unwrap();
    for doc in [PROCESSING, PUBLICATION, TERMS_V1] {
        db::consents::insert_user_consent(pool, Uuid::now_v7(), id, doc, None, None)
            .await
            .unwrap();
    }
    id
}

async fn in_pool(pool: &PgPool, id: Uuid) -> bool {
    db::pool::candidate_pool(pool, None, "backend", None)
        .await
        .unwrap()
        .iter()
        .any(|c| c.user_id == id)
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn publication_grant_gates_search(pool: PgPool) {
    let id = published_candidate(&pool, "K-GATE01").await;
    let other = published_candidate(&pool, "K-GATE02").await;
    assert!(in_pool(&pool, id).await);
    assert!(
        db::candidates::candidate_invitable(&pool, id)
            .await
            .unwrap()
            .unwrap()
            .published
    );
    let required = [
        db::consents::PROCESSING.to_owned(),
        db::consents::PUBLICATION.to_owned(),
    ];
    assert!(
        db::consents::missing_consents(&pool, id, &required)
            .await
            .unwrap()
            .is_empty()
    );

    assert_eq!(
        db::consents::revoke_user_consent(&pool, PUBLICATION, id)
            .await
            .unwrap()
            .as_deref(),
        Some("pd_publication")
    );
    assert!(
        !in_pool(&pool, id).await,
        "the visible flag alone does not publish"
    );
    assert!(in_pool(&pool, other).await);
    assert!(
        !db::candidates::candidate_invitable(&pool, id)
            .await
            .unwrap()
            .unwrap()
            .published
    );
    assert_eq!(
        db::pool::searchable_among(&pool, &[id, other])
            .await
            .unwrap(),
        [other]
    );
    let overview = db::market::category_overview(&pool).await.unwrap();
    assert_eq!(overview.iter().map(|c| c.candidates).sum::<i64>(), 1);
    let missing = db::consents::missing_consents(&pool, id, &required)
        .await
        .unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(
        (missing[0].id, missing[0].kind.as_str()),
        (PUBLICATION, "pd_publication")
    );

    let first = db::consents::grant(
        &pool,
        Uuid::now_v7(),
        id,
        PUBLICATION,
        Some("10.0.0.1"),
        None,
    )
    .await
    .unwrap();
    assert!(first.created);
    let again = db::consents::grant(&pool, Uuid::now_v7(), id, PUBLICATION, None, None)
        .await
        .unwrap();
    assert!(
        !again.created,
        "an active grant is returned, not duplicated"
    );
    assert_eq!((again.id, again.granted_at), (first.id, first.granted_at));
    assert!(in_pool(&pool, id).await);

    let history = db::consents::user_consents(&pool, id).await.unwrap();
    let publication: Vec<_> = history
        .iter()
        .filter(|c| c.kind == "pd_publication")
        .collect();
    assert_eq!(publication.len(), 2, "the revoked grant stays on record");
    assert_eq!(
        publication
            .iter()
            .filter(|c| c.revoked_at.is_none())
            .count(),
        1
    );
    assert!(
        history
            .iter()
            .any(|c| c.id == first.id && c.ip.as_deref() == Some("10.0.0.1"))
    );
}

#[sqlx::test(migrations = "../../db/migrations", fixtures("fixtures/terms_v2.sql"))]
async fn only_the_current_version_is_grantable(pool: PgPool) {
    let v1 = db::consents::document_version(&pool, TERMS_V1)
        .await
        .unwrap()
        .unwrap();
    let v2 = db::consents::document_version(&pool, TERMS_V2)
        .await
        .unwrap()
        .unwrap();
    assert!(!v1.current && v2.current);
    assert_eq!(v2.version, 2);
    let current = db::consents::current_consent_documents(&pool)
        .await
        .unwrap();
    assert!(current.iter().any(|d| d.id == TERMS_V2));
    assert!(!current.iter().any(|d| d.id == TERMS_V1));

    let video = db::consents::document_version(&pool, VIDEO)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(video.applies_to, [UserRole::Candidate]);
    let publication = db::consents::document_version(&pool, PUBLICATION)
        .await
        .unwrap()
        .unwrap();
    assert!(!publication.applies_to.contains(&UserRole::Employer));
    assert!(
        db::consents::document_version(&pool, Uuid::now_v7())
            .await
            .unwrap()
            .is_none()
    );

    let id = published_candidate(&pool, "K-VERS01").await;
    let missing = db::consents::missing_consents(&pool, id, &["terms".to_owned()])
        .await
        .unwrap();
    assert!(
        missing.is_empty(),
        "a grant of the older wording still counts"
    );
    db::consents::revoke_user_consent(&pool, TERMS_V1, id)
        .await
        .unwrap();
    let missing = db::consents::missing_consents(&pool, id, &["terms".to_owned()])
        .await
        .unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].id, TERMS_V2, "the current version is offered");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn access_log_names_the_company(pool: PgPool) {
    let candidate = published_candidate(&pool, "K-LOG001").await;
    let owner = user(&pool, UserRole::Employer, "hr-log").await;
    let company = Uuid::now_v7();
    db::companies::insert_company(&pool, company, owner, "Рога и копыта")
        .await
        .unwrap();
    for basis in ["invitation", "application"] {
        db::contacts::insert_contact_access(
            &pool,
            &db::contacts::ContactAccess {
                id: Uuid::now_v7(),
                employer_user_id: owner,
                company_id: company,
                candidate_id: candidate,
                basis,
                basis_id: Uuid::now_v7(),
                ip: Some("10.0.0.2"),
                user_agent: Some("test"),
            },
        )
        .await
        .unwrap();
    }
    let log = db::contacts::candidate_access_log(&pool, candidate, 100)
        .await
        .unwrap();
    assert_eq!(
        log.iter().map(|r| r.basis.as_str()).collect::<Vec<_>>(),
        ["application", "invitation"],
        "newest first"
    );
    assert!(
        log.iter()
            .all(|r| r.company_name.as_deref() == Some("Рога и копыта"))
    );
    assert_eq!(
        db::contacts::candidate_access_log(&pool, candidate, 1)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        db::contacts::candidate_access_log(&pool, Uuid::now_v7(), 100)
            .await
            .unwrap()
            .is_empty()
    );

    db::me::delete_company_of(&pool, owner).await.unwrap();
    let log = db::contacts::candidate_access_log(&pool, candidate, 100)
        .await
        .unwrap();
    assert_eq!(log.len(), 2, "the log outlives the company");
    assert!(log.iter().all(|r| r.company_name.is_none()));
}
