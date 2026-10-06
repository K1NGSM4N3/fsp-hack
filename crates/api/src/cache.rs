//! In-process caches of data derived from the candidate tables: the search pool of every
//! specialization with the employer cards of its candidates, the category cards and the demand
//! stats of each category (architecture.md §12.2), plus the company of each employer account,
//! read by every employer request.
//!
//! They are invalidated by a generation. A write that changes what employers can find or see on a
//! card calls [`CandidateCache::invalidate`] after its commit: the local epoch makes the next
//! request on this replica reload, and with Valkey an `INCR` reaches the other replicas within
//! [`REMOTE_TTL`]. [`HARD_TTL`] (plus at most [`RELOAD_GRACE`] while a reload runs) bounds
//! whatever does not bump: activity timestamps in scoring, a birthday crossing 18, an `INCR` lost
//! while Valkey is down.

use std::{
    future::Future,
    hash::BuildHasher,
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use arc_swap::ArcSwapOption;
use catalog::{Grade, Specialization};
use metrics::Counter;
use sqlx::PgPool;
use tokio::sync::{Notify, futures::Notified};
use uuid::Uuid;

use crate::{error::ApiResult, kv::Kv, pool::SpecPool};

pub const HARD_TTL: Duration = Duration::from_secs(30);
/// How long past [`HARD_TTL`] an entry of the current generation keeps serving while one request
/// reloads it, so the other requests don't queue behind the reload every 30 seconds.
pub const RELOAD_GRACE: Duration = Duration::from_secs(5);
pub const REMOTE_TTL: Duration = Duration::from_millis(250);
const GENERATION_KEY: &str = "cache:candidates:generation";
const NEVER: u64 = u64::MAX;
const SPECS: usize = Specialization::ALL.len();
const CATEGORIES: usize = SPECS * Grade::ALL.len();
/// Employer accounts whose company is remembered; a power of two, direct-mapped.
const COMPANY_SLOTS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    remote: u64,
    local: u64,
}

pub struct Generation {
    kv: Kv,
    local: AtomicU64,
    remote: AtomicU64,
    /// Milliseconds since `origin` of the last read of the shared counter.
    remote_read_at: AtomicU64,
    origin: Instant,
}

impl Generation {
    pub fn new(kv: Kv) -> Self {
        Self {
            kv,
            local: AtomicU64::new(0),
            remote: AtomicU64::new(0),
            remote_read_at: AtomicU64::new(NEVER),
            origin: Instant::now(),
        }
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(NEVER - 1)
    }

    pub async fn stamp(&self) -> Stamp {
        if self.kv.is_shared() {
            let now = self.now_ms();
            let read_at = self.remote_read_at.load(Ordering::Acquire);
            let due =
                read_at == NEVER || now.saturating_sub(read_at) >= REMOTE_TTL.as_millis() as u64;
            // One request refreshes the shared value; the others use the previous one meanwhile.
            if due
                && self
                    .remote_read_at
                    .compare_exchange(read_at, now, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                match self.kv.counter(GENERATION_KEY).await {
                    Ok(Some(v)) => self.remote.store(v, Ordering::Release),
                    Ok(None) => {}
                    Err(e) => tracing::warn!(
                        error = %e,
                        "cache generation read failed: writes on other replicas reach this one by the hard TTL"
                    ),
                }
            }
        }
        Stamp {
            remote: self.remote.load(Ordering::Acquire),
            local: self.local.load(Ordering::Acquire),
        }
    }

    pub async fn bump(&self) {
        self.local.fetch_add(1, Ordering::AcqRel);
        match self.kv.incr(GENERATION_KEY).await {
            Ok(Some(v)) => self.remote.store(v, Ordering::Release),
            Ok(None) => {}
            Err(e) => tracing::warn!(
                error = %e,
                "cache generation bump failed: other replicas see the write by the hard TTL"
            ),
        }
    }
}

pub struct Entry<T> {
    stamp: Stamp,
    born: Instant,
    pub value: T,
}

/// `cache_lookups_total{cache, result}`: the handles are resolved once, an increment is one
/// atomic add.
pub struct Lookups {
    hit: Counter,
    miss: Counter,
}

impl Lookups {
    fn new(cache: &'static str) -> Self {
        Self {
            hit: metrics::counter!("cache_lookups_total", "cache" => cache, "result" => "hit"),
            miss: metrics::counter!("cache_lookups_total", "cache" => cache, "result" => "miss"),
        }
    }

    pub fn record(&self, hits: usize, misses: usize) {
        if hits > 0 {
            self.hit.increment(hits as u64);
        }
        if misses > 0 {
            self.miss.increment(misses as u64);
        }
    }
}

/// Single-flight gate: one caller loads, the others wait for it instead of loading the same thing.
/// Waiters create their [`Flight::landed`] future and `enable` it before [`Flight::lead`], then
/// look at the value once more before awaiting, so a load that ends in between is not missed.
#[derive(Default)]
pub struct Flight {
    busy: AtomicBool,
    done: Notify,
}

/// Held by the caller that loads. Dropping it — after the value is stored, on an error or on
/// cancellation — wakes the waiters and lets the next caller lead.
pub struct Lead<'a>(&'a Flight);

impl Drop for Lead<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::Release);
        self.0.done.notify_waiters();
    }
}

impl Flight {
    pub fn lead(&self) -> Option<Lead<'_>> {
        // Not `then_some`: an eagerly built `Lead` would be dropped on failure and open the gate
        // held by the actual leader.
        match self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => Some(Lead(self)),
            Err(_) => None,
        }
    }

    pub fn landed(&self) -> Notified<'_> {
        self.done.notified()
    }
}

/// One cached value with single-flight loading: concurrent misses wait for one load instead of
/// all hitting the database.
pub struct Slot<T> {
    entry: ArcSwapOption<Entry<T>>,
    flight: Flight,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            entry: ArcSwapOption::empty(),
            flight: Flight::default(),
        }
    }
}

impl<T> Slot<T> {
    fn fresh(&self, stamp: Stamp) -> Option<Arc<Entry<T>>> {
        self.entry
            .load_full()
            .filter(|e| e.stamp == stamp && e.born.elapsed() < HARD_TTL)
    }

    /// Past [`HARD_TTL`] but of the current generation, and not older than [`RELOAD_GRACE`] past
    /// it: no write has invalidated it, it has only aged out.
    fn aged(&self, stamp: Stamp) -> Option<Arc<Entry<T>>> {
        self.entry
            .load_full()
            .filter(|e| e.stamp == stamp && e.born.elapsed() < HARD_TTL + RELOAD_GRACE)
    }

    /// The stamp is read before the load and kept with the value: a write committed while
    /// loading bumps the generation afterwards, so the value is never newer-labelled than its data.
    /// A value that has only aged out keeps serving everyone but the one reloading it; a value of
    /// an older generation is never served.
    pub async fn get<F, Fut>(
        &self,
        generation: &Generation,
        lookups: &Lookups,
        load: F,
    ) -> ApiResult<Arc<Entry<T>>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = ApiResult<T>>,
    {
        let mut first = true;
        loop {
            let stamp = generation.stamp().await;
            if let Some(e) = self.fresh(stamp) {
                lookups.record(usize::from(first), 0);
                return Ok(e);
            }
            if first {
                lookups.record(0, 1);
                first = false;
            }
            let mut landed = pin!(self.flight.landed());
            landed.as_mut().enable();
            if let Some(_lead) = self.flight.lead() {
                if let Some(e) = self.fresh(stamp) {
                    return Ok(e);
                }
                let born = Instant::now();
                let value = load().await?;
                let entry = Arc::new(Entry { stamp, born, value });
                self.entry.store(Some(Arc::clone(&entry)));
                return Ok(entry);
            }
            if let Some(e) = self.fresh(stamp).or_else(|| self.aged(stamp)) {
                return Ok(e);
            }
            landed.await;
        }
    }
}

struct CompanyOf {
    owner: Uuid,
    company: Uuid,
    stamp: Stamp,
    born: Instant,
}

pub struct CandidateCache {
    generation: Generation,
    pools: [Slot<SpecPool>; SPECS],
    overview: Slot<Vec<db::market::CategoryOverview>>,
    demand: [Slot<db::market::CategoryDemand>; CATEGORIES],
    companies: Box<[ArcSwapOption<CompanyOf>]>,
    hasher: std::hash::RandomState,
    pool_lookups: Lookups,
    stats_lookups: Lookups,
    company_lookups: Lookups,
    /// Employer cards, counted per card by `cards::load`.
    pub card_lookups: Lookups,
}

/// The current snapshot of each specialization, if loaded; see [`CandidateCache::current_pools`].
pub type CurrentPools = [Option<Arc<Entry<SpecPool>>>; SPECS];

impl CandidateCache {
    /// Call after the metrics recorder is installed: the counters bind to it here.
    pub fn new(kv: Kv) -> Self {
        Self {
            generation: Generation::new(kv),
            pools: std::array::from_fn(|_| Slot::default()),
            overview: Slot::default(),
            demand: std::array::from_fn(|_| Slot::default()),
            companies: (0..COMPANY_SLOTS).map(|_| ArcSwapOption::empty()).collect(),
            hasher: std::hash::RandomState::new(),
            pool_lookups: Lookups::new("pool"),
            stats_lookups: Lookups::new("category_stats"),
            company_lookups: Lookups::new("company"),
            card_lookups: Lookups::new("card"),
        }
    }

    /// The company an employer account owns. It is created with the account, never changes
    /// owner and is deleted only with the account, and account deletion invalidates the cache:
    /// an entry serves only the generation it was read in, so another replica drops it within
    /// [`REMOTE_TTL`] (the deleted session's own token is revoked at once). A missing company is
    /// not remembered.
    pub async fn company_of(&self, pg: &PgPool, owner: Uuid) -> ApiResult<Option<Uuid>> {
        let stamp = self.generation.stamp().await;
        let slot = &self.companies[self.hasher.hash_one(owner) as usize & (COMPANY_SLOTS - 1)];
        if let Some(e) = &*slot.load()
            && e.owner == owner
            && e.stamp == stamp
            && e.born.elapsed() < HARD_TTL
        {
            self.company_lookups.record(1, 0);
            return Ok(Some(e.company));
        }
        self.company_lookups.record(0, 1);
        let company = db::me::company_by_owner(pg, owner).await?.map(|c| c.id);
        if let Some(company) = company {
            slot.store(Some(Arc::new(CompanyOf {
                owner,
                company,
                stamp,
                born: Instant::now(),
            })));
        }
        Ok(company)
    }

    /// After the commit of any write to what the pool or a card is built from: profile, skills,
    /// category, privacy and search status, consents, FSP data, task submissions and reviews,
    /// proctoring uploads after the test, account deletion.
    pub async fn invalidate(&self) {
        self.generation.bump().await;
    }

    pub async fn pool(&self, pg: &PgPool, spec: Specialization) -> ApiResult<Arc<Entry<SpecPool>>> {
        self.pools[spec as usize]
            .get(&self.generation, &self.pool_lookups, || {
                SpecPool::load(pg, spec)
            })
            .await
    }

    /// The snapshots that serve requests now — fresh, or aged out and being reloaded — without
    /// loading the others.
    pub async fn current_pools(&self) -> CurrentPools {
        let stamp = self.generation.stamp().await;
        std::array::from_fn(|i| {
            let slot = &self.pools[i];
            slot.fresh(stamp).or_else(|| slot.aged(stamp))
        })
    }

    pub async fn overview(
        &self,
        pg: &PgPool,
    ) -> ApiResult<Arc<Entry<Vec<db::market::CategoryOverview>>>> {
        self.overview
            .get(&self.generation, &self.stats_lookups, || async {
                Ok(db::market::category_overview(pg).await?)
            })
            .await
    }

    /// Invitations and needs in the stats do not bump the generation: they are 30- and 90-day
    /// aggregates, so [`HARD_TTL`] is fresh enough for them.
    pub async fn demand(
        &self,
        pg: &PgPool,
        spec: Specialization,
        grade: Grade,
    ) -> ApiResult<Arc<Entry<db::market::CategoryDemand>>> {
        self.demand[spec as usize * Grade::ALL.len() + grade as usize]
            .get(&self.generation, &self.stats_lookups, || async {
                Ok(db::market::category_demand(pg, spec.slug(), grade.slug()).await?)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use super::*;

    fn generation() -> Generation {
        Generation::new(Kv::Memory(Arc::new(crate::kv::MemoryKv::new(64))))
    }

    fn lookups() -> Lookups {
        Lookups::new("test")
    }

    #[test]
    fn enum_discriminants_index_the_slots() {
        for (i, s) in Specialization::ALL.iter().enumerate() {
            assert_eq!(*s as usize, i);
        }
        for (i, g) in Grade::ALL.iter().enumerate() {
            assert_eq!(*g as usize, i);
        }
    }

    #[tokio::test]
    async fn bump_invalidates_and_reload_is_cached() {
        let g = generation();
        let slot: Slot<usize> = Slot::default();
        let loads = AtomicUsize::new(0);
        let load = || async { Ok(loads.fetch_add(1, Ordering::SeqCst)) };
        let l = lookups();
        assert_eq!(slot.get(&g, &l, load).await.unwrap().value, 0);
        assert_eq!(slot.get(&g, &l, load).await.unwrap().value, 0);
        g.bump().await;
        assert_eq!(slot.get(&g, &l, load).await.unwrap().value, 1);
        assert_eq!(slot.get(&g, &l, load).await.unwrap().value, 1);
        assert_eq!(loads.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_misses_load_once() {
        let g = Arc::new(generation());
        let slot: Arc<Slot<u32>> = Arc::new(Slot::default());
        let loads = Arc::new(AtomicUsize::new(0));
        let tasks: Vec<_> = (0..32)
            .map(|_| {
                let (g, slot, loads) = (g.clone(), slot.clone(), loads.clone());
                tokio::spawn(async move {
                    slot.get(&g, &lookups(), || async {
                        loads.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        Ok(7)
                    })
                    .await
                    .map(|e| e.value)
                })
            })
            .collect();
        for t in tasks {
            assert_eq!(t.await.unwrap().unwrap(), 7);
        }
        assert_eq!(loads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_load_is_not_cached() {
        let g = generation();
        let slot: Slot<u8> = Slot::default();
        let failed = slot
            .get(&g, &lookups(), || async {
                Err(crate::error::ApiError::from(sqlx::Error::PoolTimedOut))
            })
            .await;
        assert!(failed.is_err());
        assert_eq!(
            slot.get(&g, &lookups(), || async { Ok(3) })
                .await
                .unwrap()
                .value,
            3
        );
    }

    #[tokio::test]
    async fn aged_entry_serves_while_one_request_reloads() {
        let g = Arc::new(generation());
        let slot: Arc<Slot<u32>> = Arc::new(Slot::default());
        let stamp = g.stamp().await;
        slot.entry.store(Some(Arc::new(Entry {
            stamp,
            born: Instant::now() - HARD_TTL,
            value: 1,
        })));
        let reload = {
            let (g, slot) = (g.clone(), slot.clone());
            tokio::spawn(async move {
                slot.get(&g, &lookups(), || async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(2)
                })
                .await
                .map(|e| e.value)
            })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        let during = slot
            .get(&g, &lookups(), || async { Ok(3) })
            .await
            .unwrap()
            .value;
        assert_eq!(during, 1, "the aged value serves while the reload runs");
        assert_eq!(reload.await.unwrap().unwrap(), 2);
        assert_eq!(
            slot.get(&g, &lookups(), || async { Ok(4) })
                .await
                .unwrap()
                .value,
            2
        );

        slot.entry.store(Some(Arc::new(Entry {
            stamp,
            born: Instant::now(),
            value: 5,
        })));
        g.bump().await;
        let reload = {
            let (g, slot) = (g.clone(), slot.clone());
            tokio::spawn(async move {
                slot.get(&g, &lookups(), || async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(6)
                })
                .await
                .map(|e| e.value)
            })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        let after_write = slot
            .get(&g, &lookups(), || async { Ok(7) })
            .await
            .unwrap()
            .value;
        assert_eq!(
            after_write, 6,
            "a value of an older generation is never served"
        );
        assert_eq!(reload.await.unwrap().unwrap(), 6);
    }

    #[test]
    fn stale_entries_expire() {
        let slot: Slot<u8> = Slot::default();
        let stamp = Stamp {
            remote: 0,
            local: 0,
        };
        slot.entry.store(Some(Arc::new(Entry {
            stamp,
            born: Instant::now() - HARD_TTL,
            value: 1,
        })));
        assert!(slot.fresh(stamp).is_none());
        slot.entry.store(Some(Arc::new(Entry {
            stamp,
            born: Instant::now(),
            value: 2,
        })));
        assert!(slot.fresh(stamp).is_some());
        assert!(
            slot.fresh(Stamp {
                remote: 0,
                local: 1
            })
            .is_none()
        );
    }
}
