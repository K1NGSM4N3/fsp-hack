//! Word stems for the text-closeness feature. In production PostgreSQL ranks the candidate's
//! "about" text with `ts_rank_cd` against an OR-query of up to 40 words of the need description
//! (russian dictionary: stemming, stop words). Offline there is no database, so the same idea is
//! approximated: how many of the query's stems occur in the resume, normalized by the best
//! candidate of the specialization exactly like the API normalizes by the top rank.

use std::sync::LazyLock;

use crate::text::{fnv1a, fold_char};

/// Same cap as the API's OR-query over the need description.
pub(crate) const QUERY_TERMS: usize = 40;
const MIN_WORD_CHARS: usize = 3;
/// Russian inflection lives in the last 1–3 letters; six letters keep the stem and drop it.
const CYRILLIC_STEM: usize = 6;
const LATIN_STEM: usize = 8;
const STRIPPABLE: &str = "аяоеиыуюйь";

#[rustfmt::skip]
const STOP_WORDS: &[&str] = &[
    "что", "как", "все", "она", "так", "его", "только", "мне", "было", "вот", "меня", "еще",
    "нет", "ему", "теперь", "когда", "даже", "вдруг", "если", "уже", "или", "быть", "был",
    "него", "вас", "нибудь", "опять", "вам", "ведь", "там", "потом", "себя", "ничего", "может",
    "они", "тут", "где", "есть", "надо", "ней", "для", "тебя", "чем", "была", "сам", "чтоб",
    "без", "будто", "чего", "раз", "тоже", "себе", "под", "будет", "тогда", "кто", "этот",
    "того", "потому", "этого", "какой", "совсем", "ним", "здесь", "этом", "один", "почти",
    "мой", "тем", "чтобы", "нее", "сейчас", "были", "куда", "зачем", "всех", "никогда", "можно",
    "при", "наконец", "два", "другой", "хоть", "после", "над", "больше", "тот", "через", "эти",
    "нас", "про", "всего", "них", "какая", "много", "разве", "три", "эту", "моя", "впрочем",
    "хорошо", "свою", "этой", "перед", "иногда", "лучше", "чуть", "том", "нельзя", "такой",
    "более", "всегда", "конечно", "всю", "между", "это", "наш", "наша", "наши", "наших",
    "ваш", "также", "который", "которая", "которые", "которых", "свой", "своих",
    "the", "and", "for", "with", "you", "your", "our", "are", "will", "from", "this", "that",
    "have", "has", "was", "not", "can", "all", "any", "its", "into", "out", "who", "what",
];

static STOP: LazyLock<Vec<u32>> = LazyLock::new(|| {
    let mut v: Vec<u32> = STOP_WORDS
        .iter()
        .filter_map(|w| stem(&w.chars().collect::<Vec<_>>()))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
});

fn is_cyrillic(c: char) -> bool {
    ('а'..='я').contains(&c)
}

fn stem(word: &[char]) -> Option<u32> {
    if word.len() < MIN_WORD_CHARS {
        return None;
    }
    let cyrillic = word.iter().any(|&c| is_cyrillic(c));
    let mut end = word.len();
    while cyrillic && end > 4 && word.len() - end < 3 && STRIPPABLE.contains(word[end - 1]) {
        end -= 1;
    }
    let cap = if cyrillic { CYRILLIC_STEM } else { LATIN_STEM };
    let s: String = word[..end.min(cap)].iter().collect();
    Some(fnv1a(&s))
}

/// Stems of the text in order of appearance, stop words dropped.
fn stems(text: &str) -> impl Iterator<Item = u32> + '_ {
    let mut word: Vec<char> = Vec::with_capacity(32);
    let mut chars = text.chars().map(fold_char).chain(std::iter::once(' '));
    std::iter::from_fn(move || {
        for c in chars.by_ref() {
            if c.is_alphanumeric() {
                word.push(c);
                continue;
            }
            let s = stem(&word);
            word.clear();
            if let Some(s) = s
                && STOP.binary_search(&s).is_err()
            {
                return Some(s);
            }
        }
        None
    })
}

/// First [`QUERY_TERMS`] distinct stems, sorted for [`overlap`].
pub fn query(text: &str) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(QUERY_TERMS);
    for s in stems(text) {
        if !out.contains(&s) {
            out.push(s);
            if out.len() == QUERY_TERMS {
                break;
            }
        }
    }
    out.sort_unstable();
    out
}

/// Appends the distinct stems of a document, sorted, to `out`; returns how many were added.
pub fn document(text: &str, out: &mut Vec<u32>) -> usize {
    let start = out.len();
    out.extend(stems(text));
    out[start..].sort_unstable();
    let mut w = start;
    for r in start..out.len() {
        if w == start || out[r] != out[w - 1] {
            out[w] = out[r];
            w += 1;
        }
    }
    out.truncate(w);
    w - start
}

/// Query stems present in the (sorted) document.
pub fn overlap(query: &[u32], doc: &[u32]) -> u16 {
    query
        .iter()
        .filter(|q| doc.binary_search(q).is_ok())
        .count() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Vec<u32> {
        let mut v = Vec::new();
        document(text, &mut v);
        v
    }

    #[test]
    fn inflected_forms_share_a_stem() {
        let q = query("Разработка платёжных сервисов");
        let d = doc("Разрабатывал платежный сервис, разработку вёл сам");
        assert_eq!(overlap(&q, &d), 3);
        assert_eq!(overlap(&query("микросервисы"), &doc("Microservices")), 0);
        assert_eq!(overlap(&query("microservices"), &doc("microservice")), 1);
    }

    #[test]
    fn stop_words_and_short_words_are_ignored() {
        assert!(query("и в на для что это как the and").is_empty());
        assert_eq!(query("Go и SQL").len(), 1);
    }

    #[test]
    fn query_is_capped_and_document_deduplicated() {
        let long: String = (0..100).map(|i| format!("term{i:03} ")).collect();
        assert_eq!(query(&long).len(), QUERY_TERMS);
        let mut v = vec![7];
        assert_eq!(document("кафка кафка Кафкой", &mut v), 1);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], 7);
    }
}
