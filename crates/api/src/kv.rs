use std::{
    hash::BuildHasher,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use redis::{AsyncCommands, aio::ConnectionManager};

#[derive(Debug, thiserror::Error)]
pub enum KvError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}

/// One counted request of a fixed-window rate limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// Requests in the current window, this one included.
    pub count: u64,
    /// Until the window resets: what a refused client is told in `Retry-After`.
    pub retry_after: Duration,
}

impl Hit {
    pub fn over(&self, limit: u64) -> bool {
        self.count > limit
    }
}

/// Rate-limit counters and the access-token deny-list: Valkey in deployments,
/// a lock-free in-process table otherwise.
#[derive(Clone)]
pub enum Kv {
    Valkey(ConnectionManager),
    Memory(Arc<MemoryKv>),
}

impl Kv {
    pub async fn connect(url: Option<&str>) -> Result<Self, KvError> {
        match url {
            Some(url) => Ok(Kv::Valkey(
                ConnectionManager::new(redis::Client::open(url)?).await?,
            )),
            None => {
                tracing::warn!("VALKEY_URL not set: rate limits and deny-list are in-process");
                Ok(Kv::Memory(Arc::new(MemoryKv::new(1 << 16))))
            }
        }
    }

    /// Fixed-window counter: counts this request and says when the window resets.
    pub async fn hit(&self, key: &str, window: Duration) -> Result<Hit, KvError> {
        let secs = window.as_secs().max(1);
        match self {
            Kv::Valkey(conn) => {
                let mut conn = conn.clone();
                // One round trip, atomically. `EXPIRE … NX` sets the window only on the first
                // hit, and on every later hit repairs a key left without one by a lost reply.
                let (count, ttl): (u64, i64) = redis::pipe()
                    .atomic()
                    .incr(key, 1u64)
                    .cmd("EXPIRE")
                    .arg(key)
                    .arg(secs)
                    .arg("NX")
                    .ignore()
                    .ttl(key)
                    .query_async(&mut conn)
                    .await?;
                let left = u64::try_from(ttl).ok().filter(|t| *t > 0).unwrap_or(secs);
                Ok(Hit {
                    count,
                    retry_after: Duration::from_secs(left),
                })
            }
            Kv::Memory(mem) => {
                let now = unix_now();
                Ok(Hit {
                    count: mem.hit(key, window, now),
                    retry_after: Duration::from_secs(secs - now % secs),
                })
            }
        }
    }

    pub async fn set_flag(&self, key: &str, ttl: Duration) -> Result<(), KvError> {
        match self {
            Kv::Valkey(conn) => {
                let mut conn = conn.clone();
                let _: () = conn.set_ex(key, 1u8, ttl.as_secs().max(1)).await?;
            }
            Kv::Memory(mem) => mem.set_flag(key, ttl, unix_now()),
        }
        Ok(())
    }

    pub async fn has_flag(&self, key: &str) -> Result<bool, KvError> {
        match self {
            Kv::Valkey(conn) => {
                let mut conn = conn.clone();
                Ok(conn.exists(key).await?)
            }
            Kv::Memory(mem) => Ok(mem.has_flag(key, unix_now())),
        }
    }

    /// Shared counter (`INCR`) for state that has to agree across API replicas. `None` without
    /// Valkey: a single process keeps that counter in memory itself.
    pub async fn incr(&self, key: &str) -> Result<Option<u64>, KvError> {
        match self {
            Kv::Valkey(conn) => {
                let mut conn = conn.clone();
                Ok(Some(conn.incr(key, 1u64).await?))
            }
            Kv::Memory(_) => Ok(None),
        }
    }

    /// Current value of an [`incr`](Self::incr) counter: `Some(0)` before the first increment,
    /// `None` without Valkey.
    pub async fn counter(&self, key: &str) -> Result<Option<u64>, KvError> {
        match self {
            Kv::Valkey(conn) => {
                let mut conn = conn.clone();
                let value: Option<u64> = conn.get(key).await?;
                Ok(Some(value.unwrap_or(0)))
            }
            Kv::Memory(_) => Ok(None),
        }
    }

    pub fn is_shared(&self) -> bool {
        matches!(self, Kv::Valkey(_))
    }

    pub async fn ping(&self) -> Result<(), KvError> {
        if let Kv::Valkey(conn) = self {
            let mut conn = conn.clone();
            let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        }
        Ok(())
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Open-addressing tables of packed `AtomicU64` slots updated with CAS — no locks, fixed memory.
///
/// Counter slot: `fingerprint:24 | window:24 | count:16`. A colliding key may reset another
/// key's counter; acceptable for a single-instance fallback, Valkey is exact.
/// Flag slot: `fingerprint:32 | expires_at:32` (unix seconds), probed over `PROBE` slots so a
/// live revocation is never silently evicted while free or expired slots remain.
pub struct MemoryKv {
    counters: Box<[AtomicU64]>,
    flags: Box<[AtomicU64]>,
    mask: usize,
    hasher: std::hash::RandomState,
}

const PROBE: usize = 8;

impl MemoryKv {
    pub fn new(slots: usize) -> Self {
        assert!(slots.is_power_of_two());
        let table = || {
            (0..slots)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        Self {
            counters: table(),
            flags: table(),
            mask: slots - 1,
            hasher: std::hash::RandomState::new(),
        }
    }

    fn hash(&self, key: &str) -> u64 {
        self.hasher.hash_one(key)
    }

    pub fn hit(&self, key: &str, window: Duration, now: u64) -> u64 {
        let h = self.hash(key);
        let slot = &self.counters[h as usize & self.mask];
        let fp = (h >> 40) & 0xFF_FFFF;
        let win = (now / window.as_secs().max(1)) & 0xFF_FFFF;
        let mut current = slot.load(Ordering::Relaxed);
        loop {
            let same = current >> 40 == fp && (current >> 16) & 0xFF_FFFF == win;
            let count = if same {
                ((current & 0xFFFF) + 1).min(0xFFFF)
            } else {
                1
            };
            let next = (fp << 40) | (win << 16) | count;
            match slot.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => return count,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn set_flag(&self, key: &str, ttl: Duration, now: u64) {
        let h = self.hash(key);
        let fp = (h >> 32).max(1);
        let expires = (now + ttl.as_secs().max(1)).min(u64::from(u32::MAX));
        let packed = (fp << 32) | expires;
        let start = h as usize;
        loop {
            let mut victim: Option<(usize, u64)> = None;
            for i in 0..PROBE {
                let idx = (start + i) & self.mask;
                let current = self.flags[idx].load(Ordering::Acquire);
                let (cur_fp, cur_exp) = (current >> 32, current & 0xFFFF_FFFF);
                if current == 0 || cur_fp == fp || cur_exp <= now {
                    victim = Some((idx, current));
                    break;
                }
                if victim.is_none_or(|(_, v)| cur_exp < v & 0xFFFF_FFFF) {
                    victim = Some((idx, current));
                }
            }
            let (idx, current) = victim.expect("PROBE > 0");
            if self.flags[idx]
                .compare_exchange(current, packed, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return;
            }
        }
    }

    pub fn has_flag(&self, key: &str, now: u64) -> bool {
        let h = self.hash(key);
        let fp = (h >> 32).max(1);
        (0..PROBE).any(|i| {
            let current = self.flags[(h as usize + i) & self.mask].load(Ordering::Acquire);
            current >> 32 == fp && current & 0xFFFF_FFFF > now
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_counts_within_window_and_resets() {
        let kv = MemoryKv::new(1024);
        let w = Duration::from_secs(60);
        assert_eq!(kv.hit("a", w, 1_000), 1);
        assert_eq!(kv.hit("a", w, 1_010), 2);
        assert_eq!(kv.hit("b", w, 1_010), 1);
        assert_eq!(kv.hit("a", w, 1_100), 1);
    }

    #[tokio::test]
    async fn memory_hit_reports_the_rest_of_the_window() {
        let kv = Kv::Memory(Arc::new(MemoryKv::new(1024)));
        // Windows are aligned to multiples of their length; this one cannot roll over mid-test.
        let window = Duration::from_secs(1 << 30);
        let first = kv.hit("w", window).await.unwrap();
        let second = kv.hit("w", window).await.unwrap();
        assert_eq!((first.count, second.count), (1, 2));
        assert!(second.over(1) && !second.over(2));
        let left = second.retry_after.as_secs();
        assert!((1..=1 << 30).contains(&left), "{left}");
        assert!(left <= first.retry_after.as_secs());
    }

    /// `TEST_VALKEY_URL=redis://127.0.0.1:6379 cargo test -p api --lib kv -- --ignored`
    #[tokio::test]
    #[ignore = "needs TEST_VALKEY_URL"]
    async fn valkey_hit_reports_the_key_ttl() {
        let url = std::env::var("TEST_VALKEY_URL").expect("TEST_VALKEY_URL");
        let kv = Kv::connect(Some(&url)).await.unwrap();
        let key = format!("rl:test:{}", unix_now() ^ u64::from(std::process::id()));
        let window = Duration::from_secs(600);
        let first = kv.hit(&key, window).await.unwrap();
        let second = kv.hit(&key, window).await.unwrap();
        assert_eq!((first.count, second.count), (1, 2));
        assert!(
            (590..=600).contains(&second.retry_after.as_secs()),
            "{second:?}"
        );
        // A key that lost its expiry gets one back on the next hit instead of living forever.
        if let Kv::Valkey(conn) = &kv {
            let mut conn = conn.clone();
            let _: () = conn.persist(&key).await.unwrap();
        }
        let third = kv.hit(&key, window).await.unwrap();
        assert_eq!(third.count, 3);
        assert!(
            (590..=600).contains(&third.retry_after.as_secs()),
            "{third:?}"
        );
    }

    #[test]
    fn flags_expire() {
        let kv = MemoryKv::new(1024);
        kv.set_flag("jti", Duration::from_secs(10), 100);
        assert!(kv.has_flag("jti", 105));
        assert!(!kv.has_flag("jti", 111));
        assert!(!kv.has_flag("other", 105));
    }

    #[test]
    fn concurrent_hits_are_not_lost() {
        let kv = Arc::new(MemoryKv::new(1024));
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let kv = kv.clone();
                std::thread::spawn(move || {
                    for _ in 0..1000 {
                        kv.hit("hot", Duration::from_secs(3600), 7_200);
                    }
                })
            })
            .collect();
        threads.into_iter().for_each(|t| t.join().unwrap());
        assert_eq!(kv.hit("hot", Duration::from_secs(3600), 7_200), 8001);
    }

    #[test]
    fn many_flags_survive_in_small_table() {
        let kv = MemoryKv::new(64);
        for i in 0..32 {
            kv.set_flag(&format!("k{i}"), Duration::from_secs(100), 0);
        }
        let live = (0..32).filter(|i| kv.has_flag(&format!("k{i}"), 1)).count();
        assert!(live >= 28, "only {live} of 32 flags kept");
    }
}
