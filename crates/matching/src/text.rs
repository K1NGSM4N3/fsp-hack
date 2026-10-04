//! Text folding and dictionary matching shared by the skill, need and resume parsers.

use rustc_hash::FxHashMap;

/// Keeps a 1:1 char mapping with the input so match positions stay valid in the original.
pub(crate) fn fold_char(c: char) -> char {
    match c {
        'Ё' | 'ё' => 'е',
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        '\u{a0}' | '\u{2007}' | '\u{2009}' | '\u{202f}' => ' ',
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

pub(crate) fn fold(s: &str) -> String {
    s.chars().map(fold_char).collect()
}

pub(crate) fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_sep(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == '_'
}

fn is_cyrillic(c: char) -> bool {
    ('а'..='я').contains(&c)
}

/// Separator-insensitive key: `Spring-Boot`, `spring  boot`, `spring_boot` → `spring boot`.
pub(crate) fn key(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut gap = false;
    for c in s.chars().map(fold_char) {
        if is_sep(c) {
            gap = !out.is_empty();
        } else {
            if gap {
                out.push(' ');
                gap = false;
            }
            out.push(c);
        }
    }
    out
}

pub(crate) fn fnv1a(s: &str) -> u32 {
    s.bytes().fold(0x811c_9dc5u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

/// Russian case/number endings are built from these letters; a stem followed by at most
/// `MAX_ENDING` of them is accepted as an inflected form ("кафк|ой", "тестирован|ием").
/// Cheaper and more predictable than a stemmer, and a dictionary stem rarely continues
/// into an unrelated word using only these letters.
const ENDING_LETTERS: &str = "аяоеиыуюйьмхвг";
const STRIPPABLE: &str = "аяоеиыуюйь";
const MAX_ENDING: usize = 4;
const MIN_STEM: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tail {
    Exact,
    Inflect,
    Prefix,
}

#[derive(Debug)]
struct Word {
    chars: Box<[char]>,
    tail: Tail,
}

impl Word {
    /// Pattern word syntax: trailing `*` — any word chars may follow, `~` — force Russian
    /// inflection, `=` — exact. Unmarked Cyrillic words of 4+ letters inflect by default.
    fn parse(raw: &str) -> Word {
        let (body, forced) = match raw.chars().last() {
            Some('*') => (&raw[..raw.len() - 1], Some(Tail::Prefix)),
            Some('~') => (&raw[..raw.len() - 1], Some(Tail::Inflect)),
            Some('=') => (&raw[..raw.len() - 1], Some(Tail::Exact)),
            _ => (raw, None),
        };
        let mut chars: Vec<char> = body.chars().map(fold_char).collect();
        let tail = forced.unwrap_or(
            if chars.len() >= MIN_STEM && chars.iter().all(|&c| is_cyrillic(c)) {
                Tail::Inflect
            } else {
                Tail::Exact
            },
        );
        if tail == Tail::Inflect {
            for _ in 0..2 {
                match chars.last() {
                    Some(&c) if STRIPPABLE.contains(c) && chars.len() > MIN_STEM => {
                        chars.pop();
                    }
                    _ => break,
                }
            }
        }
        Word {
            chars: chars.into(),
            tail,
        }
    }

    fn match_at(&self, t: &[char], mut j: usize) -> Option<usize> {
        for &c in self.chars.iter() {
            if t.get(j) != Some(&c) {
                return None;
            }
            j += 1;
        }
        match self.tail {
            Tail::Exact => {}
            Tail::Prefix => {
                while j < t.len() && is_word(t[j]) {
                    j += 1;
                }
            }
            Tail::Inflect => {
                let start = j;
                while j < t.len() && is_cyrillic(t[j]) {
                    j += 1;
                }
                if j - start > MAX_ENDING
                    || !t[start..j].iter().all(|&c| ENDING_LETTERS.contains(c))
                {
                    return None;
                }
            }
        }
        Some(j)
    }
}

#[derive(Debug)]
struct Pattern<T> {
    words: Box<[Word]>,
    ambiguous: bool,
    len: usize,
    value: T,
}

impl<T> Pattern<T> {
    fn match_at(&self, t: &[char], i: usize) -> Option<usize> {
        let mut j = i;
        for (n, w) in self.words.iter().enumerate() {
            if n > 0 {
                let gap = j;
                while j < t.len() && is_sep(t[j]) {
                    j += 1;
                }
                if j == gap {
                    return None;
                }
            }
            j = w.match_at(t, j)?;
        }
        if j > i && is_word(t[j - 1]) && t.get(j).is_some_and(|&c| is_word(c)) {
            return None;
        }
        Some(j)
    }
}

/// Folded text with e-mails and links blanked out, so `go@corp.ru` or
/// `github.com/x/react-app` never produce dictionary hits.
pub(crate) struct Prepared {
    original: Vec<char>,
    folded: Vec<char>,
}

impl Prepared {
    pub(crate) fn new(s: &str) -> Self {
        let original: Vec<char> = s.chars().collect();
        let mut folded: Vec<char> = original.iter().map(|&c| fold_char(c)).collect();
        mask_links(&mut folded);
        Prepared { original, folded }
    }
}

fn mask_links(t: &mut [char]) {
    const LINK_MARKERS: [&str; 9] = [
        "@",
        "://",
        "www.",
        "t.me/",
        ".ru/",
        ".com/",
        ".io/",
        ".org/",
        "github.com",
    ];
    let mut i = 0;
    while i < t.len() {
        if t[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        while i < t.len() && !t[i].is_whitespace() {
            i += 1;
        }
        let token: String = t[start..i].iter().collect();
        if LINK_MARKERS.iter().any(|m| token.contains(m)) {
            t[start..i].fill(' ');
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Edge,
    Prose,
    Other,
}

fn side(t: &[char], mut pos: usize, forward: bool) -> Side {
    loop {
        let c = if forward {
            match t.get(pos) {
                Some(&c) => c,
                None => return Side::Edge,
            }
        } else {
            match pos.checked_sub(1) {
                Some(p) => {
                    pos = p;
                    t[p]
                }
                None => return Side::Edge,
            }
        };
        match c {
            ' ' | '\t' => {
                if forward {
                    pos += 1;
                }
            }
            c if c.is_ascii_alphanumeric() => return Side::Prose,
            _ => return Side::Other,
        }
    }
}

/// Short lowercase aliases that are also English words ("go", "rest", "react") count only
/// when capitalised, in a list, or among Russian words — not inside English prose.
fn ambiguous_ok(text: &Prepared, start: usize, end: usize) -> bool {
    if text.original[start..end].iter().any(|c| c.is_uppercase()) {
        return true;
    }
    let before = side(&text.folded, start, false);
    let after = side(&text.folded, end, true);
    matches!(
        (before, after),
        (Side::Edge, Side::Edge) | (Side::Other, _) | (_, Side::Other)
    )
}

/// Leftmost-longest, non-overlapping dictionary matcher with token boundaries.
pub(crate) struct Matcher<T> {
    patterns: Vec<Pattern<T>>,
    /// Closed key set (first chars of dictionary patterns), hence Fx hashing.
    by_first: FxHashMap<char, Vec<u32>>,
}

impl<T> Matcher<T> {
    /// A leading `?` marks an ambiguous alias (see [`ambiguous_ok`]).
    pub(crate) fn new<'s>(entries: impl IntoIterator<Item = (&'s str, T)>) -> Self {
        let mut patterns = Vec::new();
        for (spec, value) in entries {
            let (ambiguous, body) = match spec.strip_prefix('?') {
                Some(b) => (true, b),
                None => (false, spec),
            };
            let words: Box<[Word]> = body
                .split(is_sep)
                .filter(|w| !w.is_empty())
                .map(Word::parse)
                .filter(|w| !w.chars.is_empty())
                .collect();
            if words.is_empty() {
                continue;
            }
            let len = words.iter().map(|w| w.chars.len() + 1).sum();
            patterns.push(Pattern {
                words,
                ambiguous,
                len,
                value,
            });
        }
        let mut by_first: FxHashMap<char, Vec<u32>> = FxHashMap::default();
        for (i, p) in patterns.iter().enumerate() {
            by_first
                .entry(p.words[0].chars[0])
                .or_default()
                .push(i as u32);
        }
        for bucket in by_first.values_mut() {
            bucket.sort_by_key(|&i| std::cmp::Reverse(patterns[i as usize].len));
        }
        Matcher { patterns, by_first }
    }

    pub(crate) fn find(&self, text: &Prepared) -> Vec<&T> {
        let t = &text.folded;
        let mut hits = Vec::new();
        let mut i = 0;
        'scan: while i < t.len() {
            if i == 0 || !is_word(t[i - 1]) {
                if let Some(bucket) = self.by_first.get(&t[i]) {
                    for &pi in bucket {
                        let p = &self.patterns[pi as usize];
                        let Some(end) = p.match_at(t, i) else {
                            continue;
                        };
                        if p.ambiguous && !ambiguous_ok(text, i, end) {
                            continue;
                        }
                        hits.push(&p.value);
                        i = end.max(i + 1);
                        continue 'scan;
                    }
                }
            }
            i += 1;
        }
        hits
    }

    pub(crate) fn find_str(&self, s: &str) -> Vec<&T> {
        self.find(&Prepared::new(s))
    }

    pub(crate) fn first(&self, s: &str) -> Option<&T> {
        self.find_str(s).into_iter().next()
    }

    pub(crate) fn contains(&self, s: &str) -> bool {
        self.first(s).is_some()
    }
}

/// Collects distinct values in order of first appearance.
pub(crate) fn distinct<T: PartialEq + Copy>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut out = Vec::new();
    for x in items {
        if !out.contains(&x) {
            out.push(x);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(m: &Matcher<u8>, s: &str) -> Vec<u8> {
        m.find_str(s).into_iter().copied().collect()
    }

    #[test]
    fn fold_keeps_char_count() {
        let s = "Ёлка — Kafka\u{a0}İ";
        assert_eq!(fold(s).chars().count(), s.chars().count());
        assert_eq!(fold("Ёж—C#"), "еж-c#");
    }

    #[test]
    fn key_collapses_separators() {
        assert_eq!(key("  Spring-Boot "), "spring boot");
        assert_eq!(key("scikit_learn"), "scikit learn");
        assert_eq!(key("CI/CD"), "ci/cd");
    }

    #[test]
    fn boundaries_and_symbols() {
        let m = Matcher::new([
            ("go", 1u8),
            ("c++", 2),
            ("c#", 3),
            (".net", 4),
            ("node.js", 5),
        ]);
        assert_eq!(values(&m, "google, good, golang"), Vec::<u8>::new());
        assert_eq!(
            values(&m, "Go, C++17, C#, .NET, Node.js"),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(values(&m, "asp.net"), Vec::<u8>::new());
    }

    #[test]
    fn russian_inflection() {
        let m = Matcher::new([
            ("кафка", 1u8),
            ("нагрузочное тестирование", 2),
            ("реакт", 3),
        ]);
        assert_eq!(values(&m, "работал с кафкой"), vec![1]);
        assert_eq!(values(&m, "опыт нагрузочного тестирования"), vec![2]);
        assert_eq!(values(&m, "реактивный двигатель"), Vec::<u8>::new());
        assert_eq!(values(&m, "на реакте"), vec![3]);
    }

    #[test]
    fn prefix_and_flexible_separators() {
        let m = Matcher::new([("удален*", 1u8), ("spring boot", 2)]);
        assert_eq!(values(&m, "Удалёнка"), vec![1]);
        assert_eq!(values(&m, "Spring-Boot и spring  boot"), vec![2, 2]);
    }

    #[test]
    fn ambiguous_words_need_context() {
        let m = Matcher::new([("?go", 1u8), ("?rest", 2)]);
        assert_eq!(values(&m, "the rest of the team"), Vec::<u8>::new());
        assert_eq!(values(&m, "python, go, rest"), vec![1, 2]);
        assert_eq!(values(&m, "пишу на go и python"), vec![1]);
        assert_eq!(values(&m, "Go developer"), vec![1]);
        assert_eq!(values(&m, "ready to go live"), Vec::<u8>::new());
    }

    #[test]
    fn links_are_masked() {
        let m = Matcher::new([("?go", 1u8), ("react", 2)]);
        assert!(values(&m, "go@corp.ru https://github.com/x/react").is_empty());
    }
}
