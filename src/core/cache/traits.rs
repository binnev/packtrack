use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::cache::models::CacheEntry;
use crate::cache::utils::{get_cache_dir, log_hit};
use crate::tracker::TimeWindow;
use crate::utils::UtcTime;
use crate::{Result, utils};
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Cache {
    /// Get all the entries for the given url
    fn get_all(&self, url: &str) -> Vec<&CacheEntry>;

    /// Get the latest cached response.text for the given URL.
    /// Ignores the age of the entry.
    fn get(&self, url: &str) -> Option<&CacheEntry> {
        self.get_all(url)
            .into_iter()
            .max_by(|a, b| a.created.cmp(&b.created))
            .inspect(|entry| log_hit(url, entry))
    }

    /// Get the latest cached entry younger than a given age.
    fn get_younger_than(
        &self,
        url: &str,
        max_age: Duration,
    ) -> Option<&CacheEntry> {
        let now = Utc::now();
        let min_created = now - max_age;
        self.get_all(url)
            .into_iter()
            .filter(|entry| entry.created >= min_created)
            .max_by(|a, b| a.created.cmp(&b.created))
            .inspect(|entry| log_hit(url, entry))
    }

    /// Insert a cached response.text for the given URL.
    /// `mut` because the implementation must store its state in memory.
    fn insert(&mut self, url: String, text: String);

    /// Save the cache to preserve it between runs
    /// `Result` so the implementation can do IO.
    async fn save(&self) -> Result<()>;

    /// Get the size of the cache in bytes
    fn size_bytes(&self) -> Result<u64>;
}
