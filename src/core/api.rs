use crate::cache::{Cache, JsonCache};
use crate::cached_tracker::CachedTracker;
use crate::error::{Error, Result};
use crate::tracker::get_handler;
use crate::tracker::{self, TrackerContext};
use crate::tracker::{Package, PackageStatus};
use log;
use std::collections::HashMap;
use std::iter::repeat;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tokio::sync::Mutex;

/// Container for settings and runtime flags
pub struct Context {
    /// Max age for cache entries to be reused
    pub cache_seconds:      usize,
    pub filters:            Filters,
    // ----- user preferences -----
    pub default_postcode:   Option<String>,
    pub preferred_language: String,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            // TODO: Maybe make a separate Language enum which implements
            // default = "en"
            preferred_language: "en".to_string(),
            cache_seconds:      0,
            filters:            Filters::default(),
            default_postcode:   None,
        }
    }
}

#[derive(Default)]
pub struct Filters {
    /// Either a new URL, or a fragment of an existing URL
    pub url:       Option<String>,
    pub sender:    Option<String>,
    /// postal carrier e.g. DHL
    pub carrier:   Option<String>,
    pub recipient: Option<String>,
}

// TODO: This should probably be a custom error
pub struct Job {
    pub url:    String,
    pub result: Result<Package>,
}

/// Get the Tracker implementation for the given URL, and track the package.
pub async fn track_url(
    url: &str,
    cache: &Mutex<dyn Cache>,
    ctx: &Context,
) -> Job {
    let tracker = match get_handler(url) {
        Ok(tracker) => tracker,
        Err(err) => {
            return Job {
                url:    url.to_string(),
                result: Err(err),
            };
        }
    };
    let mut tracker = CachedTracker {
        tracker: tracker,
        cache:   cache,
    };
    let tracker_context = TrackerContext {
        recipient_postcode: ctx.default_postcode.as_deref(),
        language:           &ctx.preferred_language,
    };
    let result = tracker
        .track(url, ctx.cache_seconds, &tracker_context)
        .await;
    Job {
        url: url.to_string(),
        result,
    }
}

/// Track all the URLs in the URLs file.
pub async fn track_urls(urls: Vec<String>, ctx: &Context) -> Result<Vec<Job>> {
    // fire off all the tasks in parallel
    let cache = Mutex::new(JsonCache::new()?);
    let tasks: Vec<_> = urls
        .iter()
        .map(|url| track_url(url, &cache, ctx))
        .collect();
    let mut jobs = futures::future::join_all(tasks).await;
    {
        let cache = cache.lock().await;
        if cache.modified {
            cache.save().await?;
        }
    }

    if let Some(query) = &ctx.filters.recipient {
        jobs = jobs
            .into_iter()
            .filter(|job| match &job.result {
                Ok(package) => match package.recipient.as_ref() {
                    Some(recipient) => recipient
                        .to_lowercase()
                        .contains(&query.to_lowercase()),
                    None => false,
                },
                Err(err) => true, // don't remove errors
            })
            .collect();
    }
    if let Some(query) = &ctx.filters.sender {
        jobs = jobs
            .into_iter()
            .filter(|job| match &job.result {
                Ok(package) => match package.sender.as_ref() {
                    Some(sender) => sender
                        .to_lowercase()
                        .contains(&query.to_lowercase()),
                    None => false,
                },
                Err(err) => true, // don't remove errors
            })
            .collect();
    }
    if let Some(query) = &ctx.filters.carrier {
        jobs = jobs
            .into_iter()
            .filter(|job| match &job.result {
                Ok(package) => package
                    .channel
                    .to_lowercase()
                    .contains(&query.to_lowercase()),
                Err(err) => true, // don't remove errors
            })
            .collect();
    }
    Ok(jobs)
}
