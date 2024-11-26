use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::utils::UtcTime;
use crate::{settings, utils, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Cache {
    /// Get a cached response.text for the given URL
    /// `Option` because there may not be an entry for the URL.
    /// `Result` so the implementation can choose to do IO.
    fn get(&self, url: &str) -> Result<Option<&str>>;

    /// Insert a cached response.text for the given URL.
    /// `mut` because the implementation must store its state in memory.
    /// `Result` so the implementation can choose to do IO.
    fn insert(&mut self, url: String, text: String) -> Result<()>;

    /// Save the cache to preserve it between runs
    async fn save(&self) -> Result<()>;
}

pub struct JsonCache {
    contents:        HashMap<String, Vec<CacheEntry>>,
    /// max entries per url
    pub max_entries: Option<usize>,
    /// any entries older than this will not be reused
    pub max_hit_age: Option<Duration>,
    pub modified:    bool,
}
impl JsonCache {
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
impl Default for JsonCache {
    fn default() -> Self {
        Self {
            contents:    HashMap::new(),
            max_entries: Some(10),
            max_hit_age: Some(Duration::from_secs(30)),
            modified:    false,
        }
    }
}
#[async_trait]
impl Cache for JsonCache {
    fn get(&self, url: &str) -> Result<Option<&str>> {
        let now = Utc::now();
        let mut entries = self.contents.get(url);
        let min_created = self.max_hit_age.map(|max| now - max);

        // 1. maybe filter by >= min_created
        // 2. select max
        // 3. extract string
        let entry = entries
            .map(|vec| {
                vec.into_iter().filter(|entry| {
                    min_created
                        .map(|min| entry.created >= min)
                        .unwrap_or(true)
                })
            })
            .and_then(|filtered| {
                filtered
                    .into_iter()
                    .max_by(|a, b| a.created.cmp(&b.created))
            });

        match entry {
            Some(entry) => {
                let age = now - entry.created;
                log::info!(
                    "Reusing {}s old cache entry for {url}",
                    age.num_seconds()
                );
            }
            None => log::info!("Cache miss for {url}"),
        }

        Ok(entry.map(|e| e.text.as_str()))
    }
    fn insert(&mut self, url: String, text: String) -> Result<()> {
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
        Ok(())
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

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    text:    String,
    created: UtcTime,
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
        cache.insert("url".into(), "0".into())?;
        cache.insert("url".into(), "1".into())?;
        cache.insert("url".into(), "2".into())?;
        cache.insert("url".into(), "3".into())?;
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
    fn test_insert_with_no_max_values() -> Result<()> {
        let mut cache = JsonCache::default();
        assert_eq!(cache.max_entries, Some(10));
        cache.insert("url".into(), "0".into())?;
        cache.insert("url".into(), "1".into())?;
        cache.insert("url".into(), "2".into())?;
        cache.insert("url".into(), "3".into())?;
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
        Ok(())
    }

    #[test]
    fn test_get() -> Result<()> {
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

        // we should get the youngest match
        let mut cache = JsonCache {
            max_hit_age: Some(Duration::from_secs(10)),
            contents: contents.clone(),
            ..Default::default()
        };
        assert_eq!(cache.get("url")?, Some("5s ago"));

        // if no max age, we should get the same result
        let mut cache = JsonCache {
            max_hit_age: None,
            contents: contents.clone(),
            ..Default::default()
        };
        assert_eq!(cache.get("url")?, Some("5s ago"));

        // if no entries are young enough, we should get None
        let mut cache = JsonCache {
            max_hit_age: Some(Duration::from_secs(3)),
            contents: contents.clone(),
            ..Default::default()
        };
        assert_eq!(cache.get("url")?, None);

        Ok(())
    }

    #[test]
    fn test_is_modified() -> Result<()> {
        let mut cache = JsonCache::default();
        assert_eq!(cache.modified, false);
        cache.insert("url".into(), "foo".into())?;
        assert_eq!(cache.modified, true);
        Ok(())
    }
}
