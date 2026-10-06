//! A colliding public code must not fail registration: the next code is taken inside the same
//! transaction, which stays usable for the rest of the registration writes.

use api::public_code;
use sqlx::PgPool;
use uuid::Uuid;

const LOW: u128 = 0x2345_6789;

/// Ids sharing the low 30 bits, and so their first code; `upper` picks the second window.
fn id(high: u128, upper: u128) -> Uuid {
    Uuid::from_u128((high << 64) | (upper << 32) | LOW)
}

async fn user(pool: &PgPool, id: Uuid) {
    db::users::insert_user(
        pool,
        id,
        db::types::UserRole::Candidate,
        b"enc",
        id.as_bytes(),
        "h",
    )
    .await
    .unwrap();
}

async fn code_of(pool: &PgPool, id: Uuid) -> String {
    db::candidates::candidate_profile(pool, id)
        .await
        .unwrap()
        .unwrap()
        .public_code
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn registration_survives_public_code_collisions(pool: PgPool) {
    let first = id(0x0191_f000_0000_7000, 0x0101);
    let second = id(0x0191_f000_0000_7001, 0x0202);
    let third = id(0x0191_f000_0000_7002, 0x0202);
    let legacy = Uuid::now_v7();
    for u in [first, second, third, legacy] {
        user(&pool, u).await;
    }
    db::candidates::insert_candidate(&pool, legacy, "K-7F3A2C")
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let a = public_code::insert_candidate(&mut tx, first).await.unwrap();
    assert_eq!(Some(&a), public_code::candidates(first).next().as_ref());
    let b = public_code::insert_candidate(&mut tx, second)
        .await
        .unwrap();
    assert_eq!(
        public_code::candidates(second).next(),
        Some(a.clone()),
        "the first choice of `second` is taken"
    );
    assert_eq!(Some(&b), public_code::candidates(second).nth(1).as_ref());
    let c = public_code::insert_candidate(&mut tx, third).await.unwrap();
    assert!(
        public_code::candidates(third)
            .take(2)
            .all(|taken| taken == a || taken == b),
        "both id windows of `third` are taken, the code is random"
    );
    db::guardians::set_birth_date(
        &mut *tx,
        third,
        chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(),
    )
    .await
    .expect("the registration transaction is still usable after collisions");
    tx.commit().await.unwrap();

    assert_eq!(code_of(&pool, first).await, a);
    assert_eq!(code_of(&pool, second).await, b);
    assert_eq!(code_of(&pool, third).await, c);
    assert!(a != b && b != c && a != c);
    assert_eq!(
        code_of(&pool, legacy).await,
        "K-7F3A2C",
        "codes already issued stay as they are"
    );
}
