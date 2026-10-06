use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use arc_swap::ArcSwapOption;

/// Fixed-capacity ring of the latest items, lock-free for writers and readers: a writer takes a
/// sequence number with `fetch_add` and swaps its slot's `Arc`; readers load `Arc`s without
/// blocking anyone. Items carry their sequence number, so a reader racing a writer still
/// returns them newest first and never mixes in an entry older than the window.
pub struct Ring<T> {
    slots: Box<[ArcSwapOption<(u64, T)>]>,
    next: AtomicU64,
}

impl<T> Ring<T> {
    /// Holds at least one item.
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: (0..capacity.max(1))
                .map(|_| ArcSwapOption::empty())
                .collect(),
            next: AtomicU64::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Returns the sequence number given to the item.
    pub fn push(&self, item: T) -> u64 {
        let seq = self.next.fetch_add(1, Ordering::AcqRel);
        let slot = &self.slots[(seq % self.slots.len() as u64) as usize];
        slot.store(Some(Arc::new((seq, item))));
        seq
    }

    /// Up to `capacity` latest items, newest first.
    pub fn latest(&self) -> Vec<Arc<(u64, T)>> {
        let end = self.next.load(Ordering::Acquire);
        let start = end.saturating_sub(self.slots.len() as u64);
        let mut items: Vec<Arc<(u64, T)>> = self
            .slots
            .iter()
            .filter_map(ArcSwapOption::load_full)
            .filter(|item| item.0 >= start)
            .collect();
        items.sort_unstable_by_key(|item| std::cmp::Reverse(item.0));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_latest_window_newest_first() {
        let ring = Ring::new(3);
        assert!(ring.latest().is_empty());
        for i in 0..5 {
            ring.push(i);
        }
        let seen: Vec<i32> = ring.latest().iter().map(|e| e.1).collect();
        assert_eq!(seen, [4, 3, 2]);
    }

    #[test]
    fn concurrent_writers_lose_nothing_inside_the_window() {
        let ring = Arc::new(Ring::new(1_000));
        let threads: Vec<_> = (0..8)
            .map(|t| {
                let ring = ring.clone();
                std::thread::spawn(move || {
                    for i in 0..100 {
                        ring.push(t * 1_000 + i);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let items = ring.latest();
        assert_eq!(items.len(), 800);
        assert!(items.windows(2).all(|w| w[0].0 > w[1].0));
    }
}
