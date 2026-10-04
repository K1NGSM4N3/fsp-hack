use std::fmt::Display;

use rand::Rng as _;
use rand::seq::{IndexedRandom, SliceRandom};

use super::Rng;

pub(crate) fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> &'a T {
    &items[rng.random_range(0..items.len())]
}

pub(crate) fn pick_n<T: Clone>(rng: &mut Rng, items: &[T], n: usize) -> Vec<T> {
    items.choose_multiple(rng, n).cloned().collect()
}

pub(crate) fn shuffled<T>(rng: &mut Rng, mut items: Vec<T>) -> Vec<T> {
    items.shuffle(rng);
    items
}

pub(crate) fn range(rng: &mut Rng, lo: i64, hi: i64) -> i64 {
    rng.random_range(lo..=hi)
}

pub(crate) fn chance(rng: &mut Rng, p: f64) -> bool {
    rng.random_bool(p)
}

pub(crate) fn join<T: Display>(items: &[T], sep: &str) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(&item.to_string());
    }
    out
}

pub(crate) fn md_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!(
        "| {} |\n|{}\n",
        headers.join(" | "),
        "---|".repeat(headers.len())
    );
    for row in rows {
        out.push_str(&format!("| {} |\n", row.join(" | ")));
    }
    out
}

/// Shortest decimal form without trailing zeros: `2`, `0.5`, `12.25`.
pub(crate) fn fmt_number(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    let s = format!("{x:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(crate) fn normalize_text(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .replace('ё', "е")
}

pub(crate) fn parse_number(s: &str) -> Option<f64> {
    let cleaned: String = s
        .trim()
        .trim_end_matches('%')
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    cleaned.parse::<f64>().ok().filter(|x| x.is_finite())
}

/// Python `str.replace(old, new)`: non-overlapping, left to right, single pass.
pub(crate) fn py_replace(s: &str, old: &str, new: &str) -> String {
    if old.is_empty() {
        return s.to_owned();
    }
    s.replace(old, new)
}

/// Near-miss reorderings of a sequence of single-character tokens: adjacent swaps and a rotation.
pub(crate) fn perturb_sequence(answer: &str) -> Vec<String> {
    let chars: Vec<char> = answer.chars().collect();
    let mut out = Vec::new();
    for i in (0..chars.len().saturating_sub(1)).rev() {
        let mut c = chars.clone();
        c.swap(i, i + 1);
        out.push(c.into_iter().collect());
    }
    let mut c = chars;
    c.rotate_left(1);
    out.push(c.into_iter().collect());
    out
}

/// Russian noun agreement: `plural_ru(3, "запрос", "запроса", "запросов")` → "3 запроса".
pub(crate) fn plural_ru(n: i64, one: &str, few: &str, many: &str) -> String {
    let (m10, m100) = (n.abs() % 10, n.abs() % 100);
    let word = if m10 == 1 && m100 != 11 {
        one
    } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
        few
    } else {
        many
    };
    format!("{n} {word}")
}

pub(crate) const PEOPLE: &[&str] = &[
    "anna", "boris", "vera", "gleb", "dina", "egor", "zhanna", "ivan", "kira", "lev", "maria",
    "nikita", "olga", "pavel", "rita", "semyon", "taisia", "fedor",
];

pub(crate) const CITIES: &[&str] = &[
    "Москва",
    "Казань",
    "Пермь",
    "Томск",
    "Самара",
    "Сочи",
    "Омск",
    "Тверь",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_parsing_accepts_russian_format() {
        assert_eq!(parse_number(" 12,5 "), Some(12.5));
        assert_eq!(parse_number("1 024"), Some(1024.0));
        assert_eq!(parse_number("37.5%"), Some(37.5));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("NaN"), None);
    }

    #[test]
    fn text_normalization() {
        assert_eq!(normalize_text("  Ёлка   ПАЛКА \n"), "елка палка");
    }

    #[test]
    fn number_formatting() {
        assert_eq!(fmt_number(2.0), "2");
        assert_eq!(fmt_number(0.5), "0.5");
        assert_eq!(fmt_number(-3.25), "-3.25");
    }

    #[test]
    fn python_replace_is_single_pass() {
        assert_eq!(py_replace("....//....//etc", "../", ""), "../../etc");
    }
}
