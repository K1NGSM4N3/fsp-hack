//! Near-duplicate vacancy texts: word-bigram shingles compressed into a MinHash signature.
//! The fraction of equal slots in two signatures estimates the Jaccard similarity of their
//! shingle sets (standard error ≈ 0.04 at 0.6 with 128 slots), so a published vacancy is
//! compared with hundreds of recent ones by scanning fixed 512-byte rows.

use crate::text::fold;

pub const SIGNATURE_LEN: usize = 128;
/// Texts with fewer distinct word pairs are too generic to call a copy ("Go-разработчик" twice).
pub const MIN_SHINGLES: usize = 12;
/// Share of common word pairs from which two texts are the same vacancy. Reordered paragraphs or
/// changed numbers keep it above 0.8; two different vacancies of one company that share an
/// "about us" block stay at 0.2–0.4.
pub const DUPLICATE_THRESHOLD: f32 = 0.6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature([u32; SIGNATURE_LEN]);

const fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// One independent hash function per slot: `h_i(x) = splitmix(x ^ SEEDS[i])`.
const SEEDS: [u64; SIGNATURE_LEN] = {
    let mut seeds = [0u64; SIGNATURE_LEN];
    let mut i = 0;
    while i < SIGNATURE_LEN {
        seeds[i] = splitmix(i as u64 + 1);
        i += 1;
    }
    seeds
};

fn word_hash(word: &str) -> u64 {
    word.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Distinct hashes of adjacent word pairs; one-letter words (prepositions, "и") are skipped.
fn shingles(text: &str) -> Vec<u64> {
    let folded = fold(text);
    let words: Vec<u64> = folded
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().nth(1).is_some())
        .map(word_hash)
        .collect();
    let mut out: Vec<u64> = words
        .windows(2)
        .map(|w| splitmix(w[0].rotate_left(23) ^ w[1]))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

impl Signature {
    pub fn of_text(text: &str) -> Option<Signature> {
        let shingles = shingles(text);
        if shingles.len() < MIN_SHINGLES {
            return None;
        }
        let mut slots = [u32::MAX; SIGNATURE_LEN];
        for &s in &shingles {
            for (slot, seed) in slots.iter_mut().zip(SEEDS) {
                *slot = (*slot).min((splitmix(s ^ seed) >> 32) as u32);
            }
        }
        Some(Signature(slots))
    }

    pub fn similarity(&self, other: &Signature) -> f32 {
        let equal = self.0.iter().zip(&other.0).filter(|(a, b)| a == b).count();
        equal as f32 / SIGNATURE_LEN as f32
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Signature> {
        if bytes.len() != SIGNATURE_LEN * 4 {
            return None;
        }
        let mut slots = [0u32; SIGNATURE_LEN];
        for (slot, chunk) in slots.iter_mut().zip(bytes.chunks_exact(4)) {
            *slot = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        Some(Signature(slots))
    }
}

/// The text a vacancy is compared by.
pub fn vacancy_text(title: &str, description: &str) -> String {
    format!("{title}\n{description}")
}

/// Most similar signature at or above [`DUPLICATE_THRESHOLD`]: its index and similarity.
pub fn closest<'a>(
    sig: &Signature,
    others: impl IntoIterator<Item = &'a Signature>,
) -> Option<(usize, f32)> {
    others
        .into_iter()
        .enumerate()
        .map(|(i, o)| (i, sig.similarity(o)))
        .filter(|&(_, s)| s >= DUPLICATE_THRESHOLD)
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
}

/// Every signature at or above [`DUPLICATE_THRESHOLD`]: index and similarity, most similar first,
/// ties in their original order.
pub fn rank_similar(sig: &Signature, others: &[Signature]) -> Vec<(usize, f32)> {
    let mut ranked: Vec<(usize, f32)> = others
        .iter()
        .enumerate()
        .map(|(i, o)| (i, sig.similarity(o)))
        .filter(|&(_, s)| s >= DUPLICATE_THRESHOLD)
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    const GO: &str = "Go-разработчик в команду платежей. Ищем бэкенд-разработчика уровня Middle в команду \
        процессинга. Стек: Go, PostgreSQL, Kafka, gRPC, Docker. Будет плюсом опыт с Kubernetes и ClickHouse. \
        Задачи: идемпотентные платёжные API, outbox, шардирование. Зарплата 280–360 тыс. ₽, удалённо или \
        гибрид в Москве.";
    const GO_EDITED: &str = "Go-разработчик в команду платежей. Ищем бэкенд-разработчика уровня Middle в команду \
        процессинга. Стек: Go, PostgreSQL, Kafka, gRPC, Docker. Будет плюсом опыт с Kubernetes и Redis. \
        Задачи: идемпотентные платёжные API, outbox, шардирование. Зарплата 300–400 тыс. ₽, удалённо или \
        гибрид в Москве.";
    const GO_REORDERED: &str = "Задачи: идемпотентные платёжные API, outbox, шардирование. Зарплата 280–360 тыс. ₽, \
        удалённо или гибрид в Москве. Go-разработчик в команду платежей. Ищем бэкенд-разработчика уровня \
        Middle в команду процессинга. Стек: Go, PostgreSQL, Kafka, gRPC, Docker. Будет плюсом опыт с \
        Kubernetes и ClickHouse.";
    const JAVA: &str = "Senior Java-разработчик антифрода. Senior Backend: Java, Spring, PostgreSQL, Kafka, \
        системный дизайн. Нагрузка десятки тысяч RPS, проектирование правил скоринга в реальном времени. \
        От 5 лет опыта. 400 000 – 520 000 руб.";
    const ABOUT: &str = "О компании: платёжная платформа для малого бизнеса — эквайринг, выплаты, антифрод. \
        Работаем с 2015 года, офисы в Москве и Казани, ДМС с первого дня, компенсация обучения и конференций.";

    fn sig(text: &str) -> Signature {
        Signature::of_text(text).unwrap()
    }

    #[test]
    fn identical_text_is_one() {
        assert_eq!(sig(GO).similarity(&sig(GO)), 1.0);
        assert_eq!(sig(GO), sig(&GO.to_uppercase()));
    }

    #[test]
    fn light_edits_and_reordering_stay_duplicates() {
        let edited = sig(GO).similarity(&sig(GO_EDITED));
        let reordered = sig(GO).similarity(&sig(GO_REORDERED));
        assert!(edited >= DUPLICATE_THRESHOLD, "edited {edited}");
        assert!(reordered >= 0.8, "reordered {reordered}");
    }

    #[test]
    fn different_vacancies_are_far_apart() {
        let s = sig(GO).similarity(&sig(JAVA));
        assert!(s < 0.2, "{s}");
        let go = sig(&format!("{ABOUT}\n{GO}"));
        let java = sig(&format!("{ABOUT}\n{JAVA}"));
        let shared = go.similarity(&java);
        assert!(shared < DUPLICATE_THRESHOLD, "shared about block {shared}");
    }

    #[test]
    fn short_texts_have_no_signature() {
        assert!(Signature::of_text("Go-разработчик").is_none());
        assert!(Signature::of_text("Senior Java-разработчик, PostgreSQL, Kafka").is_none());
    }

    #[test]
    fn bytes_round_trip() {
        let s = sig(GO);
        let bytes = s.to_bytes();
        assert_eq!(bytes.len(), SIGNATURE_LEN * 4);
        assert_eq!(Signature::from_bytes(&bytes), Some(s));
        assert_eq!(Signature::from_bytes(&bytes[1..]), None);
    }

    #[test]
    fn closest_picks_the_best_match_above_threshold() {
        let others = [sig(JAVA), sig(GO_EDITED), sig(GO_REORDERED)];
        let (i, s) = closest(&sig(GO), &others).unwrap();
        assert_eq!(i, 2);
        assert!(s >= 0.8);
        assert_eq!(closest(&sig(GO), &others[..1]), None);
    }

    #[test]
    fn rank_similar_keeps_duplicates_only_best_first() {
        let others = [sig(GO_EDITED), sig(JAVA), sig(GO), sig(GO_REORDERED)];
        let ranked = rank_similar(&sig(GO), &others);
        let order: Vec<usize> = ranked.iter().map(|&(i, _)| i).collect();
        assert_eq!(order.first(), Some(&2), "the identical text leads");
        assert_eq!(ranked[0].1, 1.0);
        assert!(!order.contains(&1), "a different vacancy is not similar");
        assert_eq!(order.len(), 3);
        assert!(ranked.windows(2).all(|w| w[0].1 >= w[1].1));
        assert!(ranked.iter().all(|&(_, s)| s >= DUPLICATE_THRESHOLD));
        assert!(rank_similar(&sig(GO), &others[1..2]).is_empty());
    }

    #[test]
    fn rank_similar_keeps_the_original_order_of_equal_matches() {
        let others = [sig(GO), sig(JAVA), sig(GO)];
        assert_eq!(rank_similar(&sig(GO), &others), [(0, 1.0), (2, 1.0)]);
    }
}
