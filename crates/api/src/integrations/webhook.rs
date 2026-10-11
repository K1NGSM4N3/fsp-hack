//! Outbound webhook transport. The company chooses the URL, so the platform must not become a
//! proxy into its own network (SSRF): https only, no credentials in the URL, no redirects, and
//! every address the name resolves to must be public. The address check runs inside the DNS
//! resolver at connection time, so a name that re-resolves to 127.0.0.1 after validation
//! (DNS rebinding) is still refused. Hosts from `WEBHOOK_DEV_HOSTS` (the ATS mock) are exempt.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{
    Url,
    dns::{Addrs, Name, Resolve, Resolving},
    header::{CONTENT_TYPE, HeaderValue, USER_AGENT},
    redirect,
};

use super::signature::{EVENT_ID_HEADER, EVENT_TYPE_HEADER, SIGNATURE_HEADER};

pub const MAX_URL_LEN: usize = 2000;
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UrlRejection {
    #[error("not a valid absolute url")]
    Invalid,
    #[error("url too long")]
    TooLong,
    #[error("https required")]
    HttpsRequired,
    #[error("credentials in url")]
    Credentials,
    #[error("private or reserved address")]
    PrivateAddress,
    #[error("host does not resolve")]
    Unresolvable,
}

impl UrlRejection {
    pub const fn message_ru(self) -> &'static str {
        match self {
            UrlRejection::Invalid => {
                "Укажите полный адрес, например https://ats.example.ru/fsp/webhooks"
            }
            UrlRejection::TooLong => "Адрес длиннее 2000 символов",
            UrlRejection::HttpsRequired => "Нужен адрес https://",
            UrlRejection::Credentials => {
                "Логин и пароль в адресе не поддерживаются — запросы и так подписаны"
            }
            UrlRejection::PrivateAddress => {
                "Адрес ведёт во внутреннюю или служебную сеть — нужен адрес, доступный из интернета"
            }
            UrlRejection::Unresolvable => "Не удалось найти этот хост в DNS",
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{host} resolves only to private or reserved addresses")]
struct BlockedHost {
    host: String,
}

fn host_part(entry: &str) -> &str {
    match entry.rsplit_once(':') {
        Some((host, port)) if port.bytes().all(|b| b.is_ascii_digit()) => host,
        _ => entry,
    }
}

fn dev_host_matches(dev_hosts: &[String], host: &str, port: Option<u16>) -> bool {
    dev_hosts.iter().any(|entry| match entry.rsplit_once(':') {
        Some((h, p)) if p.bytes().all(|b| b.is_ascii_digit()) => {
            h == host && port.is_some_and(|port| p == port.to_string())
        }
        _ => entry == host,
    })
}

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..128).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..32).contains(&b))
        || (a == 192 && b == 168)
        || (a == 192 && b == 0 && (c == 0 || c == 2))
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224)
}

/// Globally routable unicast only: loopback, private, link-local (incl. cloud metadata
/// 169.254.169.254), CGNAT, documentation, multicast and reserved ranges are refused, and IPv6
/// forms that embed an IPv4 address are judged by that address.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => public_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return public_v4(v4);
            }
            let s = v6.segments();
            let embedded = |hi: u16, lo: u16| {
                Ipv4Addr::new((hi >> 8) as u8, hi as u8, (lo >> 8) as u8, lo as u8)
            };
            if s[0] == 0x0064 && s[1] == 0xff9b {
                return public_v4(embedded(s[6], s[7]));
            }
            if s[0] == 0x2002 {
                return public_v4(embedded(s[1], s[2]));
            }
            !(s[..6] == [0; 6]
                || v6.is_multicast()
                || (s[0] & 0xfe00) == 0xfc00
                || (s[0] & 0xffc0) == 0xfe80
                || (s[0] & 0xffc0) == 0xfec0
                || (s[0] == 0x2001 && s[1] == 0x0db8)
                || (s[0] == 0x0100 && s[1..4] == [0; 3]))
        }
    }
}

/// Static checks done when the URL is saved and again before every delivery.
pub fn check_url(raw: &str, dev_hosts: &[String]) -> Result<Url, UrlRejection> {
    let raw = raw.trim();
    if raw.len() > MAX_URL_LEN {
        return Err(UrlRejection::TooLong);
    }
    let url = Url::parse(raw).map_err(|_| UrlRejection::Invalid)?;
    let host = url
        .host_str()
        .ok_or(UrlRejection::Invalid)?
        .to_ascii_lowercase();
    if !url.username().is_empty() || url.password().is_some() {
        return Err(UrlRejection::Credentials);
    }
    let trusted = dev_host_matches(dev_hosts, &host, url.port_or_known_default());
    match url.scheme() {
        "https" => {}
        "http" if trusted => {}
        "http" => return Err(UrlRejection::HttpsRequired),
        _ => return Err(UrlRejection::Invalid),
    }
    if !trusted
        && let Some(ip) = literal_ip(&host)
        && !is_public(ip)
    {
        return Err(UrlRejection::PrivateAddress);
    }
    Ok(url)
}

fn literal_ip(host: &str) -> Option<IpAddr> {
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .ok()
}

struct GuardedResolver {
    dev_hosts: Arc<[String]>,
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_ascii_lowercase();
        let trusted = self.dev_hosts.iter().any(|h| host_part(h) == host);
        Box::pin(async move {
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await?
                .filter(|a| trusted || is_public(a.ip()))
                .collect();
            if addrs.is_empty() {
                return Err(Box::new(BlockedHost { host }) as _);
            }
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

/// One HTTP attempt as it goes to the delivery log.
#[derive(Debug, Clone)]
pub struct Attempt {
    pub status: Option<u16>,
    pub latency_ms: i32,
    pub error: Option<String>,
}

impl Attempt {
    pub fn ok(&self) -> bool {
        self.status.is_some_and(|s| (200..300).contains(&s))
    }
}

pub struct SignedRequest<'a> {
    pub event_id: &'a str,
    pub event_type: &'a str,
    pub signature: &'a str,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct WebhookSender {
    client: reqwest::Client,
    dev_hosts: Arc<[String]>,
}

impl WebhookSender {
    pub fn new(dev_hosts: Vec<String>) -> Result<Self, reqwest::Error> {
        let dev_hosts: Arc<[String]> = dev_hosts.into();
        let client = reqwest::Client::builder()
            .dns_resolver(GuardedResolver {
                dev_hosts: dev_hosts.clone(),
            })
            .redirect(redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(TIMEOUT)
            .pool_max_idle_per_host(2)
            .build()?;
        Ok(Self { client, dev_hosts })
    }

    pub fn check_url(&self, raw: &str) -> Result<Url, UrlRejection> {
        check_url(raw, &self.dev_hosts)
    }

    /// Save-time feedback for names: the same rule the resolver applies on every connection.
    pub async fn check_resolution(&self, url: &Url) -> Result<(), UrlRejection> {
        let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
            return Err(UrlRejection::Invalid);
        };
        if literal_ip(&host).is_some()
            || dev_host_matches(&self.dev_hosts, &host, url.port_or_known_default())
        {
            return Ok(());
        }
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
            .await
            .map_err(|_| UrlRejection::Unresolvable)?
            .collect();
        if addrs.is_empty() {
            Err(UrlRejection::Unresolvable)
        } else if addrs.iter().all(|a| is_public(a.ip())) {
            Ok(())
        } else {
            Err(UrlRejection::PrivateAddress)
        }
    }

    pub async fn post(&self, raw_url: &str, req: SignedRequest<'_>) -> Attempt {
        let started = Instant::now();
        let elapsed = |started: Instant| started.elapsed().as_millis().min(i32::MAX as u128) as i32;
        let url = match self.check_url(raw_url) {
            Ok(url) => url,
            Err(e) => {
                return Attempt {
                    status: None,
                    latency_ms: 0,
                    error: Some(e.message_ru().to_owned()),
                };
            }
        };
        let header = |v: &str| HeaderValue::from_str(v).unwrap_or(HeaderValue::from_static(""));
        let sent = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "FSP-Talents-Webhooks/1.0")
            .header(EVENT_ID_HEADER, header(req.event_id))
            .header(EVENT_TYPE_HEADER, header(req.event_type))
            .header(SIGNATURE_HEADER, header(req.signature))
            .body(req.body)
            .send()
            .await;
        match sent {
            Ok(res) => {
                let status = res.status().as_u16();
                Attempt {
                    status: Some(status),
                    latency_ms: elapsed(started),
                    error: (!(200..300).contains(&status)).then(|| format!("Ответ HTTP {status}")),
                }
            }
            Err(e) => Attempt {
                status: None,
                latency_ms: elapsed(started),
                error: Some(describe(&e)),
            },
        }
    }
}

fn describe(e: &reqwest::Error) -> String {
    let mut root: &(dyn std::error::Error + 'static) = e;
    while let Some(next) = root.source() {
        root = next;
    }
    if e.is_timeout() {
        format!("Нет ответа за {} с", TIMEOUT.as_secs())
    } else if root.is::<BlockedHost>() {
        UrlRejection::PrivateAddress.message_ru().to_owned()
    } else if e.is_connect() {
        format!("Не удалось подключиться: {root}")
    } else {
        root.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev() -> Vec<String> {
        vec!["ats-mock:8082".into(), "127.0.0.1".into()]
    }

    #[test]
    fn public_and_private_addresses() {
        for ip in [
            "93.184.216.34",
            "8.8.8.8",
            "2a00:1450:4010:c05::71",
            "77.88.55.242",
        ] {
            assert!(is_public(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "198.18.0.1",
            "192.0.2.10",
            "::1",
            "::",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "2001:db8::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::a00:1",
            "2002:c0a8:0101::1",
        ] {
            assert!(!is_public(ip.parse().unwrap()), "{ip}");
        }
        assert!(is_public("::ffff:8.8.8.8".parse().unwrap()));
        assert!(is_public("172.32.0.1".parse().unwrap()));
    }

    #[test]
    fn url_rules() {
        let d = dev();
        assert!(check_url("https://ats.example.ru/hooks/fsp", &d).is_ok());
        assert!(check_url("https://93.184.216.34:8443/x", &d).is_ok());
        assert_eq!(
            check_url("http://ats.example.ru/hooks", &d),
            Err(UrlRejection::HttpsRequired)
        );
        assert_eq!(
            check_url("https://user:pass@ats.example.ru/", &d),
            Err(UrlRejection::Credentials)
        );
        for private in [
            "https://127.0.0.1/",
            "https://10.0.0.5/hook",
            "https://169.254.169.254/latest/meta-data",
            "https://[::1]:8443/",
            "https://[::ffff:192.168.0.1]/",
        ] {
            assert_eq!(
                check_url(private, &[]),
                Err(UrlRejection::PrivateAddress),
                "{private}"
            );
        }
        assert_eq!(
            check_url("ftp://ats.example.ru/", &d),
            Err(UrlRejection::Invalid)
        );
        assert_eq!(check_url("not a url", &d), Err(UrlRejection::Invalid));
        assert_eq!(
            check_url(&format!("https://a.ru/{}", "x".repeat(MAX_URL_LEN)), &d),
            Err(UrlRejection::TooLong)
        );
    }

    #[test]
    fn dev_hosts_allow_http_and_private_only_for_themselves() {
        let d = dev();
        assert!(check_url("http://ats-mock:8082/webhooks", &d).is_ok());
        assert_eq!(
            check_url("http://ats-mock:9000/webhooks", &d),
            Err(UrlRejection::HttpsRequired),
            "the allowlisted port only"
        );
        assert!(check_url("http://127.0.0.1:41234/webhooks", &d).is_ok());
        assert_eq!(
            check_url("http://127.0.0.2/webhooks", &d),
            Err(UrlRejection::HttpsRequired)
        );
        assert_eq!(
            check_url("https://127.0.0.2/webhooks", &d),
            Err(UrlRejection::PrivateAddress)
        );
        assert_eq!(
            check_url("http://ats-mock:8082/", &[]),
            Err(UrlRejection::HttpsRequired),
            "nothing is trusted by default"
        );
    }

    #[tokio::test]
    async fn resolver_refuses_names_that_point_inside() {
        let guarded = GuardedResolver {
            dev_hosts: Arc::from(Vec::<String>::new()),
        };
        let blocked = guarded.resolve("localhost".parse().unwrap()).await;
        assert!(blocked.is_err(), "localhost resolves to loopback only");

        let trusted = GuardedResolver {
            dev_hosts: Arc::from(vec!["localhost:8082".to_owned()]),
        };
        let addrs: Vec<SocketAddr> = trusted
            .resolve("localhost".parse().unwrap())
            .await
            .unwrap()
            .collect();
        assert!(addrs.iter().all(|a| a.ip().is_loopback()));
    }

    #[tokio::test]
    async fn delivery_to_a_private_name_is_refused() {
        let sender = WebhookSender::new(Vec::new()).unwrap();
        let attempt = sender
            .post(
                "https://localhost:9/hook",
                SignedRequest {
                    event_id: "e",
                    event_type: "integration.test",
                    signature: "t=1,v1=00",
                    body: b"{}".to_vec(),
                },
            )
            .await;
        assert!(!attempt.ok());
        assert_eq!(
            attempt.error.as_deref(),
            Some(UrlRejection::PrivateAddress.message_ru())
        );
    }
}
