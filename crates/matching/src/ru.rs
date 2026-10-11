//! Russian formatting helpers for explanations.

/// `forms` = [one, few, many]: 1 достижение, 2 достижения, 5 достижений.
pub(crate) fn plural(n: usize, forms: [&str; 3]) -> &str {
    let (n10, n100) = (n % 10, n % 100);
    if n10 == 1 && n100 != 11 {
        forms[0]
    } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
        forms[1]
    } else {
        forms[2]
    }
}

/// `250000` → `250 000 ₽` with non-breaking spaces.
pub(crate) fn rub(v: i64) -> String {
    let digits = v.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + 8);
    if v < 0 {
        out.push('-');
    }
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(c);
    }
    out.push_str("\u{a0}₽");
    out
}

pub(crate) fn days_ago(days: i64) -> String {
    match days {
        ..=0 => "сегодня".to_owned(),
        1 => "вчера".to_owned(),
        d => format!("{d} дн. назад"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_forms() {
        let f = ["навык", "навыка", "навыков"];
        assert_eq!(plural(1, f), "навык");
        assert_eq!(plural(3, f), "навыка");
        assert_eq!(plural(11, f), "навыков");
        assert_eq!(plural(22, f), "навыка");
        assert_eq!(plural(25, f), "навыков");
    }

    #[test]
    fn money() {
        assert_eq!(rub(250_000), "250\u{a0}000\u{a0}₽");
        assert_eq!(rub(1_500_000), "1\u{a0}500\u{a0}000\u{a0}₽");
        assert_eq!(rub(900), "900\u{a0}₽");
    }
}
