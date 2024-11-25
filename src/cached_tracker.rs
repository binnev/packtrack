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
        let cached = cache
            .get(url)
            .await?
            .map(|s| s.to_owned());
        drop(cache); // allows other async threads to continue

        let text = match cached {
            Some(text) => text.to_owned(),
            None => {
                let text = self.tracker.get_raw(url).await?;
                self.cache
                    .lock()
                    .await
                    .insert(url.to_owned(), text.clone())
                    .await?;
                text
            }
        };
        let package = self.tracker.parse(text)?;
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::test_cache::TestCache;

    struct MockTracker;
    #[async_trait]
    impl Tracker for MockTracker {
        fn can_handle(&self, url: &str) -> bool {
            true
        }
        async fn get_raw(&self, url: &str) -> Result<String> {
            Ok("I am a potato".into())
        }
        fn parse(&self, text: String) -> Result<Package> {
            Err("I am a potato".into())
        }
    }

    #[tokio::test]
    async fn test_sync_mutex_in_async_tasks() {
        let mutex = Mutex::new(0);
        async fn add(m: &Mutex<usize>) {
            let mut number = m.lock().await;
            *number += 1;
        };
        let mut tasks = vec![];
        for _ in (0..3) {
            tasks.push(add(&mutex))
        }
        assert_eq!(tasks.len(), 3);
        let x = futures::future::join_all(tasks).await;
        assert_eq!(*mutex.lock().await, 3);
    }
}
