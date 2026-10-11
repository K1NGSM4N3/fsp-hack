//! Corporate domain check: the company's verified login email and its website live on the same
//! domain. Domains are compared in lowercase ASCII, so `пример.рф` and `xn--e1afmkfd.xn--p1ai`
//! are the same site.

use serde::{Deserialize, Serialize};

/// Free mailbox providers: an address there says nothing about the employer. Sorted for binary
/// search; subdomains (`corp.mail.ru`) count as the provider too.
const PUBLIC_MAIL: &[&str] = &[
    "aol.com",
    "bk.ru",
    "e1.ru",
    "gmail.com",
    "gmx.com",
    "gmx.de",
    "googlemail.com",
    "hotmail.com",
    "i.ua",
    "icloud.com",
    "inbox.ru",
    "internet.ru",
    "lenta.ru",
    "list.ru",
    "live.com",
    "mac.com",
    "mail.com",
    "mail.ru",
    "mail.ua",
    "me.com",
    "msn.com",
    "myrambler.ru",
    "ngs.ru",
    "outlook.com",
    "pm.me",
    "proton.me",
    "protonmail.com",
    "qip.ru",
    "rambler.ru",
    "ro.ru",
    "tuta.io",
    "tutanota.com",
    "ukr.net",
    "xmail.ru",
    "ya.ru",
    "yahoo.com",
    "yandex.by",
    "yandex.com",
    "yandex.kz",
    "yandex.ru",
    "yandex.ua",
    "zoho.com",
];

/// Zones where unrelated owners get sibling subdomains: second-level public suffixes and site
/// builders. Matching on them would let `acme.github.io` vouch for any `*.github.io` address.
/// Sorted for binary search.
const SHARED_ZONES: &[&str] = &[
    "blogspot.com",
    "co.il",
    "co.jp",
    "co.uk",
    "com.br",
    "com.by",
    "com.cn",
    "com.kz",
    "com.ru",
    "com.tr",
    "com.ua",
    "firebaseapp.com",
    "github.io",
    "gitlab.io",
    "herokuapp.com",
    "kiev.ua",
    "msk.ru",
    "msk.su",
    "narod.ru",
    "net.ru",
    "netlify.app",
    "org.kz",
    "org.ru",
    "org.uk",
    "pages.dev",
    "pp.ru",
    "spb.ru",
    "spb.su",
    "tilda.ws",
    "ucoz.ru",
    "vercel.app",
    "web.app",
    "wixsite.com",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DomainCheck {
    /// `domain` is the shared registrable part, e.g. `corp.ru` for `hr@corp.ru` and `jobs.corp.ru`.
    Verified {
        domain: String,
    },
    EmailUnverified,
    PublicMail {
        email_domain: String,
    },
    NoWebsite {
        email_domain: String,
    },
    BadWebsite {
        email_domain: String,
    },
    Mismatch {
        email_domain: String,
        site: String,
    },
}

impl DomainCheck {
    pub fn verified_domain(&self) -> Option<&str> {
        match self {
            DomainCheck::Verified { domain } => Some(domain),
            _ => None,
        }
    }

    /// What the employer sees: the result and, when it failed, how to fix it.
    pub fn hint(&self) -> String {
        match self {
            DomainCheck::Verified { domain } => format!(
                "Почта для входа и сайт компании на одном домене — {}. Кандидаты видят у ваших вакансий и приглашений отметку «Домен подтверждён».",
                display_domain(domain)
            ),
            DomainCheck::EmailUnverified => {
                "Подтвердите email для входа — после этого домен проверится автоматически.".into()
            }
            DomainCheck::PublicMail { email_domain } => format!(
                "Вы входите с адреса на {} — это публичный почтовый сервис, он не подтверждает связь с компанией. Отметка появляется у аккаунта с корпоративной почтой на домене сайта.",
                display_domain(email_domain)
            ),
            DomainCheck::NoWebsite { email_domain } => format!(
                "Укажите сайт компании в профиле. Если он на домене {} (поддомены тоже подходят), домен подтвердится сразу.",
                display_domain(email_domain)
            ),
            DomainCheck::BadWebsite { .. } => {
                "Не получилось разобрать адрес сайта. Укажите его в виде https://example.ru.".into()
            }
            DomainCheck::Mismatch { email_domain, site } => format!(
                "Почта для входа на {}, а сайт — {}. Домен подтверждается, когда они совпадают; поддомены подходят, например jobs.{}.",
                display_domain(email_domain),
                display_domain(site),
                display_domain(email_domain)
            ),
        }
    }
}

pub fn check_domain(email: &str, email_verified: bool, website: Option<&str>) -> DomainCheck {
    let mail = match email_domain(email) {
        Some(d) if email_verified => d,
        _ => return DomainCheck::EmailUnverified,
    };
    if is_public_mail(&mail) {
        return DomainCheck::PublicMail { email_domain: mail };
    }
    let Some(site) = website.map(str::trim).filter(|w| !w.is_empty()) else {
        return DomainCheck::NoWebsite { email_domain: mail };
    };
    let Some(host) = website_host(site) else {
        return DomainCheck::BadWebsite { email_domain: mail };
    };
    match shared_base(&mail, &host) {
        Some(domain) => DomainCheck::Verified { domain },
        None => DomainCheck::Mismatch {
            email_domain: mail,
            site: host,
        },
    }
}

/// One domain equals the other or is its subdomain, and the shorter one is a registrable domain
/// rather than a TLD or a shared zone.
fn shared_base(a: &str, b: &str) -> Option<String> {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let nested = long == short
        || long
            .strip_suffix(short)
            .is_some_and(|prefix| prefix.ends_with('.'));
    (nested && !is_shared_zone(short)).then(|| short.to_owned())
}

/// `a.b.c` → `a.b.c`, `b.c`, `c`.
fn parents(domain: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(Some(domain), |d| d.split_once('.').map(|(_, rest)| rest))
}

pub fn is_public_mail(domain: &str) -> bool {
    parents(domain).any(|d| PUBLIC_MAIL.binary_search(&d).is_ok())
}

fn is_shared_zone(domain: &str) -> bool {
    !domain.contains('.') || SHARED_ZONES.binary_search(&domain).is_ok()
}

pub fn email_domain(email: &str) -> Option<String> {
    let (local, domain) = email.trim().rsplit_once('@')?;
    if local.is_empty() {
        return None;
    }
    ascii_domain(domain).filter(|d| d.contains('.'))
}

/// Host of a website address without scheme, credentials, port, path and a leading `www.`.
pub fn website_host(url: &str) -> Option<String> {
    let mut rest = url.trim();
    if let Some((scheme, after)) = rest.split_once("://") {
        if scheme.is_empty() || !scheme.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        rest = after;
    } else if let Some(after) = rest.strip_prefix("//") {
        rest = after;
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let mut authority = &rest[..end];
    if let Some((_, host)) = authority.rsplit_once('@') {
        authority = host;
    }
    let host = match authority.rsplit_once(':') {
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => authority,
    };
    let host = ascii_domain(host)?;
    let host = match host.strip_prefix("www.") {
        Some(bare) => bare.to_owned(),
        None => host,
    };
    let tld_has_letter = host
        .rsplit('.')
        .next()
        .is_some_and(|tld| tld.bytes().any(|b| b.is_ascii_lowercase()));
    (host.contains('.') && tld_has_letter).then_some(host)
}

/// Lowercase ASCII form of a domain; Unicode labels become punycode
/// (`пример.рф` → `xn--e1afmkfd.xn--p1ai`).
pub fn ascii_domain(raw: &str) -> Option<String> {
    let raw = raw.trim().trim_end_matches('.');
    if raw.is_empty() || raw.len() > 253 {
        return None;
    }
    let mut out = String::with_capacity(raw.len() + 8);
    for (i, label) in raw.split('.').enumerate() {
        if i > 0 {
            out.push('.');
        }
        let lower = label.to_lowercase();
        let ascii = if lower.is_ascii() {
            lower
        } else {
            format!("xn--{}", punycode_encode(&lower)?)
        };
        if !valid_label(&ascii) {
            return None;
        }
        out.push_str(&ascii);
    }
    Some(out)
}

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Punycode labels back to Unicode for display; anything undecodable stays as is.
pub fn display_domain(ascii: &str) -> String {
    ascii
        .split('.')
        .map(|label| {
            label
                .strip_prefix("xn--")
                .and_then(punycode_decode)
                .unwrap_or_else(|| label.to_owned())
        })
        .collect::<Vec<_>>()
        .join(".")
}

// RFC 3492 bootstring parameters for punycode.
const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;

fn adapt(delta: u32, points: u32, first: bool) -> u32 {
    let mut delta = if first { delta / DAMP } else { delta / 2 };
    delta += delta / points;
    let mut k = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + (BASE - T_MIN + 1) * delta / (delta + SKEW)
}

fn threshold(k: u32, bias: u32) -> u32 {
    if k <= bias {
        T_MIN
    } else if k >= bias + T_MAX {
        T_MAX
    } else {
        k - bias
    }
}

fn encode_digit(d: u32) -> char {
    let b = if d < 26 {
        b'a' + d as u8
    } else {
        b'0' + (d - 26) as u8
    };
    char::from(b)
}

fn decode_digit(b: u8) -> Option<u32> {
    match b {
        b'a'..=b'z' => Some(u32::from(b - b'a')),
        b'A'..=b'Z' => Some(u32::from(b - b'A')),
        b'0'..=b'9' => Some(u32::from(b - b'0') + 26),
        _ => None,
    }
}

fn punycode_encode(label: &str) -> Option<String> {
    let input: Vec<u32> = label.chars().map(u32::from).collect();
    let mut out: String = label.chars().filter(char::is_ascii).collect();
    let basic = out.len() as u32;
    let mut handled = basic;
    if basic > 0 {
        out.push('-');
    }
    let (mut n, mut delta, mut bias) = (INITIAL_N, 0u32, INITIAL_BIAS);
    while (handled as usize) < input.len() {
        let m = input.iter().copied().filter(|&c| c >= n).min()?;
        delta = delta.checked_add((m - n).checked_mul(handled + 1)?)?;
        n = m;
        for &c in &input {
            if c < n {
                delta = delta.checked_add(1)?;
            }
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = threshold(k, bias);
                    if q < t {
                        break;
                    }
                    out.push(encode_digit(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                out.push(encode_digit(q));
                bias = adapt(delta, handled + 1, handled == basic);
                delta = 0;
                handled += 1;
            }
        }
        delta = delta.checked_add(1)?;
        n = n.checked_add(1)?;
    }
    Some(out)
}

fn punycode_decode(encoded: &str) -> Option<String> {
    let (basic, digits) = match encoded.rfind('-') {
        Some(i) => (&encoded[..i], &encoded[i + 1..]),
        None => ("", encoded),
    };
    if !basic.is_ascii() {
        return None;
    }
    let mut out: Vec<char> = basic.chars().collect();
    let (mut n, mut i, mut bias) = (INITIAL_N, 0u32, INITIAL_BIAS);
    let mut bytes = digits.bytes().peekable();
    while bytes.peek().is_some() {
        let old_i = i;
        let mut w = 1u32;
        let mut k = BASE;
        loop {
            let digit = decode_digit(bytes.next()?)?;
            i = i.checked_add(digit.checked_mul(w)?)?;
            let t = threshold(k, bias);
            if digit < t {
                break;
            }
            w = w.checked_mul(BASE - t)?;
            k += BASE;
        }
        let len = out.len() as u32 + 1;
        bias = adapt(i - old_i, len, old_i == 0);
        n = n.checked_add(i / len)?;
        i %= len;
        out.insert(i as usize, char::from_u32(n)?);
        i += 1;
    }
    Some(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified(email: &str, site: &str) -> Option<String> {
        check_domain(email, true, Some(site))
            .verified_domain()
            .map(str::to_owned)
    }

    #[test]
    fn lists_are_sorted() {
        assert!(PUBLIC_MAIL.windows(2).all(|w| w[0] < w[1]));
        assert!(SHARED_ZONES.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn same_domain_and_subdomains_match() {
        assert_eq!(
            verified("hr@severny-kod.ru", "https://severny-kod.ru").as_deref(),
            Some("severny-kod.ru")
        );
        assert_eq!(
            verified("HR@Severny-Kod.RU", "http://www.severny-kod.ru/about?x=1").as_deref(),
            Some("severny-kod.ru")
        );
        assert_eq!(
            verified("hr@corp.ru", "jobs.corp.ru").as_deref(),
            Some("corp.ru")
        );
        assert_eq!(
            verified("hr@msk.corp.ru", "https://corp.ru:8443/").as_deref(),
            Some("corp.ru")
        );
    }

    #[test]
    fn lookalikes_do_not_match() {
        assert_eq!(verified("hr@evilcorp.ru", "https://corp.ru"), None);
        assert_eq!(verified("hr@corp.ru.evil.com", "https://corp.ru"), None);
        assert_eq!(verified("hr@corp.ru", "https://corp.ru.evil.com"), None);
        assert!(matches!(
            check_domain("hr@corp.ru", true, Some("https://other.ru")),
            DomainCheck::Mismatch { .. }
        ));
    }

    #[test]
    fn public_mail_never_counts() {
        for email in [
            "hr.company@gmail.com",
            "jobs@yandex.ru",
            "a@mail.ru",
            "a@bk.ru",
            "a@inbox.ru",
            "a@list.ru",
            "a@rambler.ru",
            "a@outlook.com",
            "a@icloud.com",
            "a@proton.me",
            "a@corp.mail.ru",
        ] {
            assert!(
                matches!(
                    check_domain(email, true, Some("https://mail.ru")),
                    DomainCheck::PublicMail { .. }
                ),
                "{email}"
            );
        }
        assert_eq!(verified("hr@gmail.com", "https://gmail.com"), None);
    }

    #[test]
    fn shared_zones_and_bare_tlds_are_not_a_base() {
        assert_eq!(verified("hr@acme.github.io", "https://github.io"), None);
        assert_eq!(verified("hr@shop.com.ru", "https://com.ru"), None);
        assert_eq!(verified("hr@corp.ru", "https://ru"), None);
        assert_eq!(
            verified("hr@acme.com.ru", "https://acme.com.ru").as_deref(),
            Some("acme.com.ru")
        );
    }

    #[test]
    fn missing_or_broken_inputs() {
        assert_eq!(
            check_domain("hr@corp.ru", false, Some("corp.ru")),
            DomainCheck::EmailUnverified
        );
        assert_eq!(
            check_domain("hr@corp.ru", true, None),
            DomainCheck::NoWebsite {
                email_domain: "corp.ru".into()
            }
        );
        assert_eq!(
            check_domain("hr@corp.ru", true, Some("  ")),
            DomainCheck::NoWebsite {
                email_domain: "corp.ru".into()
            }
        );
        assert!(matches!(
            check_domain("hr@corp.ru", true, Some("https://exa mple.ru")),
            DomainCheck::BadWebsite { .. }
        ));
        assert_eq!(website_host("http://127.0.0.1:3000"), None);
        assert_eq!(website_host("javascript:alert(1)"), None);
    }

    #[test]
    fn punycode_matches_rfc_examples() {
        assert_eq!(punycode_encode("пример").as_deref(), Some("e1afmkfd"));
        assert_eq!(punycode_encode("рф").as_deref(), Some("p1ai"));
        assert_eq!(
            punycode_encode("испытание").as_deref(),
            Some("80akhbyknj4f")
        );
        assert_eq!(punycode_encode("münchen").as_deref(), Some("mnchen-3ya"));
        for label in ["пример", "рф", "испытание", "münchen", "москва", "café-bar"]
        {
            let encoded = punycode_encode(label).unwrap();
            assert_eq!(punycode_decode(&encoded).as_deref(), Some(label));
        }
    }

    #[test]
    fn idn_and_punycode_forms_are_the_same_domain() {
        assert_eq!(
            ascii_domain("Пример.РФ").as_deref(),
            Some("xn--e1afmkfd.xn--p1ai")
        );
        assert_eq!(
            verified("hr@пример.рф", "https://xn--e1afmkfd.xn--p1ai/").as_deref(),
            Some("xn--e1afmkfd.xn--p1ai")
        );
        assert_eq!(
            verified("hr@XN--E1AFMKFD.xn--p1ai", "https://www.пример.рф").as_deref(),
            Some("xn--e1afmkfd.xn--p1ai")
        );
        assert_eq!(display_domain("xn--e1afmkfd.xn--p1ai"), "пример.рф");
        assert_eq!(display_domain("corp.ru"), "corp.ru");
    }

    #[test]
    fn hints_name_both_sides() {
        let hint = check_domain(
            "employer@demo.fsp",
            true,
            Some("https://severny-kod.example"),
        )
        .hint();
        assert!(hint.contains("demo.fsp") && hint.contains("severny-kod.example"));
        assert!(
            check_domain("hr@пример.рф", true, Some("пример.рф"))
                .hint()
                .contains("пример.рф")
        );
    }
}
