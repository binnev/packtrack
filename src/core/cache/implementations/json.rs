use std::os::unix::fs::MetadataExt;
use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::cache::models::CacheEntry;
use crate::cache::traits::Cache;
use crate::cache::utils::{get_cache_dir, log_hit};
use crate::tracker::TimeWindow;
use crate::utils::UtcTime;
use crate::{Result, utils};
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::fs::metadata;

#[derive(Default)]
pub struct JsonCache {
    contents:        HashMap<String, Vec<CacheEntry>>,
    /// max entries per url
    pub max_entries: Option<usize>,
    /// any entries older than this will not be reused
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
    pub fn get_file() -> Result<PathBuf> {
        Ok(get_cache_dir()?.join("packtrack-cache.json"))
    }
}
#[async_trait]
impl Cache for JsonCache {
    fn get_all_urls(&self) -> Vec<String> {
        self.contents.keys().cloned().collect()
    }
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
    fn remove(&mut self, url: &str) -> Vec<CacheEntry> {
        let entries = self
            .contents
            .remove(url)
            .unwrap_or_default();
        if entries.len() > 0 {
            log::info!(
                "Removed {} from cache which had {} entries",
                url,
                entries.len()
            );
            self.modified = true;
        }
        entries
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
    fn size_bytes(&self) -> Result<u64> {
        let cache_file = Self::get_file()?;
        if cache_file.exists() {
            Ok(metadata(cache_file)?.size())
        } else {
            Ok(0)
        }
    }
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
        assert!(
            cache
                .get("url")
                .unwrap()
                .text
                .eq("text")
        );

        cache.insert("url".into(), "text2".into());
        cache.insert("url".into(), "text3".into());
        assert!(
            cache
                .get("url")
                .unwrap()
                .text
                .eq("text3")
        );
    }
    #[test]
    fn test_remove() {
        let now = Utc::now();
        let mut cache = JsonCache::default();
        cache.insert("url".into(), "text".into());
        cache.insert("url".into(), "text2".into());
        cache.insert("url2".into(), "text3".into());

        let removed = cache.remove("url");
        assert_eq!(removed.len(), 2);
        assert_eq!(cache.contents.len(), 1);
    }
    #[test]
    fn test_prune() {
        let now = Utc::now();
        let mut cache = JsonCache::default();
        cache.insert("url".into(), "text".into());
        cache.insert("url".into(), "text2".into());
        cache.insert("url2".into(), "text3".into());
        cache.insert("url3".into(), "text4".into());

        let keep: Vec<String> = vec!["url2".into(), "url3".into()];
        let removed = cache.prune(&keep);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed, ["url"]);
        assert_eq!(cache.contents.len(), 2);

        let mut removed = cache.prune(&vec![]);
        removed.sort();
        assert_eq!(removed, ["url2", "url3"]);
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
        assert!(
            cache
                .get_younger_than("url", Duration::from_secs(10))
                .unwrap()
                .text
                .eq("5s ago")
        );

        // if no max age, we should get the same result
        assert!(
            cache
                .get("url")
                .unwrap()
                .text
                .eq("5s ago")
        );

        // if no entries are young enough, we should get None
        assert!(
            cache
                .get_younger_than("url", Duration::from_secs(3))
                .is_none()
        );
    }

    #[test]
    fn test_is_modified() {
        let mut cache = JsonCache::default();
        assert_eq!(cache.modified, false);
        cache.insert("url".into(), "foo".into());
        assert_eq!(cache.modified, true);
    }
}
