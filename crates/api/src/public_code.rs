//! The candidate's anonymous handle shown to employers before contacts are granted, «K-7Q3M2X».
//! Issued once at registration and never recomputed: shortlists, invitations and ATS payloads
//! carry it, so codes issued under the earlier scheme (six hex digits) stay as they are.

use rand::Rng;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::error::{ApiResult, InternalError};

/// Crockford base32: no I, L, O, U, so a code survives being read aloud or retyped. Six characters
/// give 2^30 codes; six hex digits gave 2^24 and collided a few times per 10 000 candidates.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const BITS: u32 = 30;
/// Each attempt at 10 000 candidates collides with probability 10^-5, so the cap is never reached.
pub const ATTEMPTS: usize = 8;

fn encode(value: u32) -> String {
    let mut code = String::with_capacity(8);
    code.push_str("K-");
    for i in (0..6).rev() {
        code.push(char::from(ALPHABET[((value >> (5 * i)) & 31) as usize]));
    }
    code
}

/// Codes to try in order: two disjoint 30-bit windows of the UUIDv7 random tail (deterministic for
/// a given id), then random ones.
pub fn candidates(id: Uuid) -> impl Iterator<Item = String> {
    let tail = id.as_u128();
    let mask = (1u128 << BITS) - 1;
    [0, 32]
        .into_iter()
        .map(move |shift| encode(((tail >> shift) & mask) as u32))
        .chain(std::iter::repeat_with(|| {
            encode(rand::rng().random::<u32>() >> (32 - BITS))
        }))
}

/// Creates the candidate row with the first free code. `ON CONFLICT DO NOTHING` keeps the
/// surrounding registration transaction usable after a collision, so no savepoint is needed.
pub async fn insert_candidate(tx: &mut PgConnection, user_id: Uuid) -> ApiResult<String> {
    for code in candidates(user_id).take(ATTEMPTS) {
        if db::candidates::insert_candidate(&mut *tx, user_id, &code).await? == 1 {
            return Ok(code);
        }
    }
    Err(InternalError::PublicCodeExhausted(ATTEMPTS).into())
}

/// A code as a person types it — «k-7q3m2x», «K7Q3M2X», with spaces around — in its stored form,
/// or `None` if it is not a code at all.
pub fn normalize(raw: &str) -> Option<String> {
    let upper = raw.trim().to_ascii_uppercase();
    let body = upper
        .strip_prefix("K-")
        .or_else(|| upper.strip_prefix('K'))?;
    (body.len() == 6 && body.bytes().all(|b| ALPHABET.contains(&b))).then(|| format!("K-{body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_look_like_k_and_six_unambiguous_characters() {
        let codes: Vec<String> = candidates(Uuid::now_v7()).take(5).collect();
        for code in &codes {
            assert_eq!(code.len(), 8, "{code}");
            assert!(code.starts_with("K-"));
            assert!(code[2..].bytes().all(|b| ALPHABET.contains(&b)), "{code}");
            assert!(!code[2..].contains(['I', 'L', 'O', 'U']));
        }
    }

    #[test]
    fn typed_codes_normalize() {
        assert_eq!(normalize(" k-7q3m2x ").as_deref(), Some("K-7Q3M2X"));
        assert_eq!(normalize("K7Q3M2X").as_deref(), Some("K-7Q3M2X"));
        assert_eq!(normalize("K-00A1F3").as_deref(), Some("K-00A1F3"));
        for bad in [
            "",
            "K-",
            "K-7Q3M2",
            "K-7Q3M2XX",
            "7Q3M2X",
            "K-7Q3M2!",
            "0191f000-0000-7000-8000-000000000001",
        ] {
            assert_eq!(normalize(bad), None, "{bad}");
        }
    }

    #[test]
    fn first_codes_come_from_the_id() {
        let id = Uuid::from_u128((5u128 << 32) | 1);
        let codes: Vec<String> = candidates(id).take(2).collect();
        assert_eq!(codes, ["K-000001", "K-000005"]);
        assert_eq!(candidates(id).next(), candidates(id).next());
        assert_eq!(encode((1 << BITS) - 1), "K-ZZZZZZ");
    }
}
