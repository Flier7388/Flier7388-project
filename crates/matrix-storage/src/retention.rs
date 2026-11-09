//! Pure least-recently-used eviction decision logic for the on-disk
//! media cache, decoupled from any actual filesystem I/O so it can be
//! unit-tested without touching disk.

/// A single cached media blob tracked by [`RetentionPolicy`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// Opaque cache key, e.g. the `mxc://` URI this blob was fetched from.
    pub key: String,
    /// Size of the cached blob on disk, in bytes.
    pub size_bytes: u64,
}

/// A disk budget and the least-recently-used eviction policy applied to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    max_total_bytes: u64,
}

impl RetentionPolicy {
    /// Creates a policy that keeps the cache within `max_total_bytes`.
    #[must_use]
    pub const fn new(max_total_bytes: u64) -> Self {
        Self { max_total_bytes }
    }

    /// The configured disk budget, in bytes.
    #[must_use]
    pub const fn max_total_bytes(&self) -> u64 {
        self.max_total_bytes
    }

    /// Selects which of `entries` (ordered **oldest-accessed first**,
    /// i.e. the next LRU eviction candidates come first) must be
    /// evicted so that admitting `incoming_bytes` more data keeps the
    /// cache within [`max_total_bytes`](Self::max_total_bytes).
    ///
    /// Returns the indices into `entries` to evict, in the order they
    /// should be removed. An empty return value means the incoming blob
    /// fits without evicting anything.
    #[must_use]
    pub fn evictions_for(&self, entries: &[CacheEntry], incoming_bytes: u64) -> Vec<usize> {
        let current_total: u64 = entries.iter().map(|entry| entry.size_bytes).sum();
        let mut projected_total = current_total.saturating_add(incoming_bytes);

        if projected_total <= self.max_total_bytes {
            return Vec::new();
        }

        let mut victims = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            if projected_total <= self.max_total_bytes {
                break;
            }
            projected_total = projected_total.saturating_sub(entry.size_bytes);
            victims.push(index);
        }
        victims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, size_bytes: u64) -> CacheEntry {
        CacheEntry {
            key: key.to_owned(),
            size_bytes,
        }
    }

    #[test]
    fn admits_without_eviction_when_under_budget() {
        let policy = RetentionPolicy::new(1_000);
        let entries = [entry("a", 100), entry("b", 200)];
        assert_eq!(policy.evictions_for(&entries, 50), Vec::<usize>::new());
    }

    #[test]
    fn evicts_oldest_entries_first_until_incoming_blob_fits() {
        let policy = RetentionPolicy::new(1_000);
        let entries = [
            entry("oldest", 400),
            entry("middle", 400),
            entry("newest", 100),
        ];
        // 900 already cached + 300 incoming = 1200 > 1000; evicting
        // "oldest" (400) brings the projected total to 800, which fits.
        assert_eq!(policy.evictions_for(&entries, 300), vec![0]);
    }

    #[test]
    fn evicts_everything_if_the_incoming_blob_alone_exceeds_the_budget() {
        let policy = RetentionPolicy::new(1_000);
        let entries = [entry("a", 100)];
        assert_eq!(policy.evictions_for(&entries, 10_000), vec![0]);
    }

    #[test]
    fn empty_cache_needs_no_eviction_for_a_blob_that_fits() {
        let policy = RetentionPolicy::new(1_000);
        assert_eq!(policy.evictions_for(&[], 999), Vec::<usize>::new());
    }
}
