use crate::cache::Cache;
use crate::tracker::{Package, Tracker, TrackerContext};
use crate::{Error, Result};
use tokio::sync::Mutex;

/// Composed type with pluggable tracker + cache handlers. Orchestrates:
/// - Fetching a raw value from either the Tracker or the Cache
/// - Parsing the raw value with Tracker
pub struct CachedTracker<'a> {
    pub tracker: Box<dyn Tracker>,
    pub cache:   &'a Mutex<dyn Cache>,
}
impl<'a> CachedTracker<'a> {
    pub async fn track(
        &mut self,
        url: &str,
        cache_seconds: usize,
        use_cache: bool,
        ctx: &'a TrackerContext<'_>,
    ) -> Result<Package> {
        if use_cache {
            match self
                .get_cached(url, cache_seconds)
                .await
            {
                Ok(Some(package)) => return Ok(package),
                Err(err) => log::warn!(
                    "Error loading from cache: {err}. Getting a fresh value."
                ),
                Ok(None) => log::info!(
                    "No cache entry found for {url}. Getting a fresh value."
                ),
            }
        }
        self.get_fresh(url, ctx).await
    }

    async fn get_fresh(
        &mut self,
        url: &str,
        ctx: &'a TrackerContext<'_>,
    ) -> Result<Package> {
        let text = match self.tracker.get_raw(url, ctx).await {
            Ok(text) => text,
            // If we receive a client error (4xx) it is sometimes because we
            // tried to use the user's home postcode on a package for which the
            // user is not the recipient (for example, a return). This results
            // in a 404 from the carrier API because the postcodes don't match.
            // In this case, we want to retry _without_ the user's default
            // postcode, because then we will at least get a response.
            Err(Error::Reqwest(err))
                if err
                    .status()
                    .is_some_and(|s| s.is_client_error()) =>
            {
                log::warn!(
                    "Bad response: {err}, trying again without default postcode..."
                );
                let mut ctx = ctx.clone();
                ctx.recipient_postcode = None;
                self.tracker.get_raw(url, &ctx).await?
            }
            Err(err) => return Err(err),
        };
        self.cache
            .lock()
            .await
            .insert(url.to_owned(), text.clone());
        let package = self.tracker.parse(text)?;
        Ok(package)
    }

    async fn get_cached(
        &mut self,
        url: &str,
        cache_seconds: usize,
    ) -> Result<Option<Package>> {
        let cache = self.cache.lock().await;
        let cached = cache.get(url).cloned();
        drop(cache); // allows other async threads to use it

        if let Some(entry) = cached {
            match self.tracker.parse(entry.text.clone()) {
                Err(err) => {
                    return Err(
                        format!(
                        "Couldn't parse cache entry to package! url: {url}, cache entry: {entry:?}, error: {err:?}").into()
                    );
                }
                Ok(package) => {
                    let age = entry.age().num_seconds().unsigned_abs() as usize;

                    // Always cache packages with a final status, because they
                    // will receive no more updates.
                    if package.status.is_final() {
                        log::info!(
                            "Reusing {age}s old cache entry for delivered {} {} from url {url}",
                            package.channel,
                            package.barcode,
                        );
                        return Ok(Some(package));
                    }

                    // Cache undelivered packages if the entry is young enough
                    if age <= cache_seconds {
                        log::info!(
                            "Reusing {age}s old cache entry for undelivered {} {} from url {url}",
                            package.channel,
                            package.barcode,
                        );
                        return Ok(Some(package));
                    }
                }
            }
        }
        Ok(None)
    }
}
