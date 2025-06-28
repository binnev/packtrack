use std::time::Duration;

use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::Result;
use crate::cache::{Cache, JsonCache};
use crate::tracker::{Package, PackageStatus, Tracker};

/// Composed type with pluggable tracker + cache handlers.
pub struct CachedTracker<'a> {
    pub tracker: Box<dyn Tracker>,
    pub cache:   &'a Mutex<dyn Cache>,
}
impl CachedTracker<'_> {
    pub async fn track(
        &mut self,
        url: &str,
        cache_seconds: usize,
    ) -> Result<Package> {
        let cache = self.cache.lock().await;
        let cached = cache.get(url).cloned();
        drop(cache); // allows other async threads to continue

        if let Some(entry) = cached {
            let package = self.tracker.parse(entry.text.clone())?;
            let age = entry.age().num_seconds().unsigned_abs() as usize;

            // Always cache delivered packages
            if package.status() == PackageStatus::Delivered {
                log::info!(
                    "Reusing {age}s old cache entry for delivered {} {} from url {url}",
                    package.channel,
                    package.barcode,
                );
                return Ok(package);
            }

            // Cache undelivered packages if the entry is young enough, and the
            // cache is enabled
            if age <= cache_seconds {
                log::info!(
                    "Reusing {age}s old cache entry for undelivered {} {} from url {url}",
                    package.channel,
                    package.barcode,
                );
                return Ok(package);
            }
        }

        // Fallback: fetch a fresh one
        let text = self.tracker.get_raw(url).await?;
        self.cache
            .lock()
            .await
            .insert(url.to_owned(), text.clone());
        let package = self.tracker.parse(text)?;
        Ok(package)
    }
}

pub struct CacheContext {
    /// TODO: setting this to 0 supersedes use_cache below.
    max_age_s: usize,
    /// Can be deactivated with the `--no-cache` flag
    use_cache: bool,
}
