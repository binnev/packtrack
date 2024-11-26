use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::cache::{Cache, JsonCache};
use crate::tracker::{Package, Tracker};
use crate::Result;

/// Composed type with pluggable tracker + cache handlers.
pub struct CachedTracker<'a> {
    pub tracker: Box<dyn Tracker>,
    pub cache:   &'a Mutex<dyn Cache>,
}
impl CachedTracker<'_> {
    pub async fn track(&mut self, url: &str) -> Result<Package> {
        let cache = self.cache.lock().await;
        let cached = cache.get(url)?.map(|s| s.to_owned());
        drop(cache); // allows other async threads to continue

        let text = match cached {
            Some(text) => text.to_owned(),
            None => {
                let text = self.tracker.get_raw(url).await?;
                self.cache
                    .lock()
                    .await
                    .insert(url.to_owned(), text.clone())?;
                text
            }
        };
        let package = self.tracker.parse(text)?;
        Ok(package)
    }
}
