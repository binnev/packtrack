use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::settings::Settings;
use crate::tracker::TimeWindow;
use crate::utils::UtcTime;
use crate::{settings, utils, Result};
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
}
#[derive(Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub text:    String,
    pub created: UtcTime,
}
impl CacheEntry {
    pub fn age(&self) -> TimeDelta {
        Utc::now() - self.created
    }
}

fn log_hit(url: &str, entry: &CacheEntry) {
    log::debug!(
        "Reusing {}s old cache entry for {url}",
        entry.age().num_seconds()
    )
}
#[derive(Default)]
pub struct JsonCache {
    contents:        HashMap<String, Vec<CacheEntry>>,
    /// max entries per url
    pub max_entries: Option<usize>,
    /// any entries older than this will not be reused
    pub modified:    bool,
}
impl JsonCache {
    pub fn from_settings(settings: Settings) -> Result<Self> {
        Ok(Self {
            contents: Self::load_contents()?,
            max_entries: Some(settings.cache_max_entries),
            ..Default::default()
        })
    }
    pub fn new() -> Result<Self> {
        Ok(Self {
            contents: Self::load_contents()?,
            ..Default::default()
        })
    }
    pub fn with_max_entries(max_entries: usize) -> Result<Self> {
        Ok(Self {
            contents: Self::load_contents()?,
            max_entries: Some(max_entries),
            ..Default::default()
        })
    }
    // RAII: load from file when instantiating
    fn load_contents() -> Result<HashMap<String, Vec<CacheEntry>>> {
        #[cfg(test)]
        return Ok(HashMap::new()); // don't load from file in tests

        let cache_file = Self::get_file()?;
        let contents = utils::load_json(&cache_file)?;
        log::info!("Loaded JSON cache from {cache_file:?}");
        Ok(contents)
    }
    fn get_file() -> Result<PathBuf> {
        Ok(get_cache_dir()?.join("packtrack-cache.json"))
    }
}
#[async_trait]
impl Cache for JsonCache {
    fn get_all(&self, url: &str) -> Vec<&CacheEntry> {
        self.contents
            .get(url)
            .map(|v| v.iter().collect())
            .unwrap_or(vec![])
    }
    fn insert(&mut self, url: String, text: String) {
        let entry = CacheEntry {
            created: Utc::now(),
            text:    text,
        };
        self.contents
            .entry(url.clone())
            .and_modify(|e| {
                e.push(entry.clone());
                // maintain max length
                if self
                    .max_entries
                    .map(|max| e.len() > max)
                    .unwrap_or(false)
                {
                    e.remove(0);
                }
            })
            .or_insert(vec![entry]);
        log::info!("Inserted new cache entry for {url}");
        self.modified = true;
    }
    // Save to file
    async fn save(&self) -> Result<()> {
        #[cfg(test)]
        return Ok(()); // don't write to file in tests

        let cache_file = Self::get_file()?;
        utils::save_json(&cache_file, &self.contents)?;
        log::info!("Saved JSON cache to {cache_file:?}");
        Ok(())
    }
}

fn get_cache_dir() -> Result<PathBuf> {
    let dirs = utils::project_dirs()?;
    let cache_dir = dirs.cache_dir();
    Ok(cache_dir.to_owned())
}

#[cfg(test)]
mod tests {

    use std::fmt::format;

    use super::*;

    #[test]
    fn test_insert_with_max_values() -> Result<()> {
        let mut cache = JsonCache::with_max_entries(2)?;
        assert_eq!(cache.max_entries, Some(2));
        cache.insert("url".into(), "0".into());
        cache.insert("url".into(), "1".into());
        cache.insert("url".into(), "2".into());
        cache.insert("url".into(), "3".into());
        let hits = cache.contents.get("url").unwrap();
        assert_eq!(hits.len(), 2);
        let entries: Vec<&str> = cache
            .contents
            .get("url")
            .unwrap()
            .iter()
            .map(|e| e.text.as_str())
            .collect();
        // only the 2 most recent ones should be kept
        assert_eq!(entries, vec!["2", "3"]);
        Ok(())
    }
    #[test]
    fn test_insert_with_no_max_values() {
        let mut cache = JsonCache::default();
        assert_eq!(cache.max_entries, None);
        cache.insert("url".into(), "0".into());
        cache.insert("url".into(), "1".into());
        cache.insert("url".into(), "2".into());
        cache.insert("url".into(), "3".into());
        let hits = cache.contents.get("url").unwrap();
        assert_eq!(hits.len(), 4);
        let entries: Vec<&str> = cache
            .contents
            .get("url")
            .unwrap()
            .iter()
            .map(|e| e.text.as_str())
            .collect();
        // only the 2 most recent ones should be kept
        assert_eq!(entries, vec!["0", "1", "2", "3"]);
    }

    #[test]
    fn test_get() {
        let now = Utc::now();
        let mut cache = JsonCache::default();

        assert!(cache.get("url").is_none());

        cache.insert("url".into(), "text".into());
        assert!(cache
            .get("url")
            .unwrap()
            .text
            .eq("text"));

        cache.insert("url".into(), "text2".into());
        cache.insert("url".into(), "text3".into());
        assert!(cache
            .get("url")
            .unwrap()
            .text
            .eq("text3"));
    }
    #[test]
    fn test_get_younger_than() {
        let now = Utc::now();
        let contents = HashMap::from([(
            "url".into(),
            [20, 5, 10]
                .iter()
                .map(|delta| CacheEntry {
                    created: now - Duration::from_secs(*delta),
                    text:    format!("{delta}s ago"),
                })
                .collect(),
        )]);
        let mut cache = JsonCache {
            contents: contents.clone(),
            ..Default::default()
        };

        // we should get the youngest match
        assert!(cache
            .get_younger_than("url", Duration::from_secs(10))
            .unwrap()
            .text
            .eq("5s ago"));

        // if no max age, we should get the same result
        assert!(cache
            .get("url")
            .unwrap()
            .text
            .eq("5s ago"));

        // if no entries are young enough, we should get None
        assert!(cache
            .get_younger_than("url", Duration::from_secs(3))
            .is_none());
    }

    #[test]
    fn test_is_modified() {
        let mut cache = JsonCache::default();
        assert_eq!(cache.modified, false);
        cache.insert("url".into(), "foo".into());
        assert_eq!(cache.modified, true);
    }
}
