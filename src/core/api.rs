use crate::cache::{Cache, JsonCache};
use crate::cached_tracker::CachedTracker;
use crate::error::{Error, Result};
use crate::tracker;
use crate::tracker::get_handler;
use crate::tracker::{Package, PackageStatus};
use crate::urls;
use chrono::{DateTime, Local, TimeZone};
use enum_iterator::all;
use log;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::repeat;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tokio::sync::Mutex;

/// Container for settings and runtime flags
pub struct Context {
    // ----- cache -----
    /// Max age for cache entries to be reused
    pub cache_seconds:  usize,
    // ----- display -----
    /// e.g. None for CLI printing. "json" for JSON output (that can be piped
    /// to a file or other programs)
    pub display_format: Option<String>,
    pub filters:        Filters,
}
pub struct Filters {
    /// Either a new URL, or a fragment of an existing URL
    pub url:       Option<String>,
    pub sender:    Option<String>,
    /// postal carrier e.g. DHL
    pub carrier:   Option<String>,
    pub recipient: Option<String>,
}

// TODO: This should probably be a custom error
struct Job {
    url:    String,
    result: Result<Package>,
}

// -- Public API
pub async fn track(ctx: &Context) -> Result<()> {
    let start = Instant::now();
    let urls = urls::filter(ctx.filters.url.as_deref())?;
    track_urls(urls, ctx).await?;
    log::info!("track_all took {:?}", start.elapsed());
    Ok(())
}

// -- Internals

/// Get the Tracker implementation for the given URL, and track the package.
async fn track_url(url: &str, cache: &Mutex<dyn Cache>, ctx: &Context) -> Job {
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
    let result = tracker
        .track(url, ctx.cache_seconds)
        .await;
    Job {
        url: url.to_string(),
        result,
    }
}

/// Track all the URLs in the URLs file.
async fn track_urls(urls: Vec<String>, ctx: &Context) -> Result<()> {
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

    // sort the results by status / error
    let mut errors: Vec<Job> = vec![];
    let mut jobs_by_status: HashMap<PackageStatus, Vec<Package>> =
        HashMap::new();
    for job in jobs {
        match &job.result {
            Ok(package) => {
                let status = package.status();
                jobs_by_status
                    .entry(status)
                    .and_modify(|e| e.push(package.clone()))
                    .or_insert(vec![package.clone()]);
            }
            Err(err) => errors.push(job),
        }
    }
    // sort by time
    for (status, packages) in jobs_by_status.iter_mut() {
        if status == &PackageStatus::Delivered {
            packages.sort_by(|a, b| a.delivered.cmp(&b.delivered));
        }
        if status == &PackageStatus::InTransit {
            packages.sort_by(|a, b| a.eta.cmp(&b.eta));
            packages.sort_by(|a, b| {
                let a_time = a
                    .eta
                    .or(a.eta_window.as_ref().map(|w| w.start));
                let b_time = b
                    .eta
                    .or(b.eta_window.as_ref().map(|w| w.start));
                a_time.cmp(&b_time)
            });
        }
    }

    // display successful results
    for status in all::<PackageStatus>() {
        let mut packages = jobs_by_status
            .entry(status.clone())
            .or_insert(vec![]);
        let separator = match status {
            PackageStatus::Delivered => "\n".to_owned(),
            PackageStatus::InTransit => {
                format!("\n{}\n", "-".repeat(80))
            }
        };
        heading(&status);
        let s = packages
            .iter()
            .map(|package| format!("{package}"))
            .collect::<Vec<_>>()
            .join(&separator);
        println!("{s}");
    }

    // display errors
    heading(&"errors");
    let separator = format!("\n{}\n", "-".repeat(80));
    let s = errors
        .iter()
        .map(|job| format!("{}\n{:?}", job.url, job.result))
        .collect::<Vec<_>>()
        .join(&separator);
    println!("{s}");

    Ok(())
}

fn heading(s: &dyn Display) {
    println!("{}", "=".repeat(80));
    let text = format!(" {s} ");
    let text = spaced(text).to_uppercase();
    let text = format!("{text:^80}");
    println!("{}", text);
    println!("{}", "=".repeat(80));
}

/// "hello" -> "h e l l o"
fn spaced(s: String) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Display the date as "Fri 22 Nov" or "Today"
pub fn display_date<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    let is_today = local.date_naive() == Local::now().date_naive();
    if is_today {
        "Today".into()
    } else {
        local.format("%a %d %b").to_string()
    }
}

/// Display a datetime as "Fri 22 Nov 12:00"
pub fn display_time<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    format!("{} {}", display_date(dt), local.format("%H:%M"))
}
