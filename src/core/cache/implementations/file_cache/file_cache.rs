use std::fs::metadata;
use std::os::unix::fs::MetadataExt;
use std::{collections::HashMap, path::PathBuf};

use chrono::Utc;

use crate::{
    Result,
    cache::{
        Cache,
        implementations::file_cache::serializer::{
            CacheEntrySerializer, JsonCacheEntrySerializer,
        },
        models::CacheEntry,
    },
    file_handler::{FileHandler, TextFileHandler},
};

/// Cache which stores its entries as a `HashMap<String, CacheEntry>` in memory,
/// and as a text file on disk. Handles different file formats.
pub struct FileCache {
    path:            PathBuf,
    contents:        HashMap<String, Vec<CacheEntry>>,
    pub max_entries: Option<usize>,
    pub modified:    bool,
    file_handler:    Box<dyn FileHandler>,
    serializer:      Box<dyn CacheEntrySerializer>,
}

impl FileCache {
    /// RAII -- instantiating the struct also loads the cache from file.
    pub fn new(path: PathBuf) -> Result<Self> {
        #[allow(unused_mut)]
        let mut file_handler: Box<dyn FileHandler> = Box::new(TextFileHandler);

        // Use a mock file handler in tests to prevent tests doing IO
        #[cfg(test)]
        {
            use crate::file_handler::MockFileHandler;
            file_handler = Box::new(MockFileHandler);
        }

        let serializer = Self::select_serializer(&path)?;
        let text = file_handler.load(&path)?;
        let contents = serializer.deserialize(&text)?;
        Ok(Self {
            path,
            contents,
            file_handler,
            serializer,
            max_entries: None,
            modified: false,
        })
    }
    fn select_serializer(
        file: &PathBuf,
    ) -> Result<Box<dyn CacheEntrySerializer>> {
        if !file.exists() {
            return Err(
                format!("File {} does not exist!", file.display()).into()
            );
        }
        let ext = match file.extension() {
            Some(s) => s.to_str().ok_or(format!(
                "Filename {} is invalid unicode?!",
                file.display()
            ))?,
            None => {
                return Err(format!(
                    "File {} has no extension!",
                    file.display()
                )
                .into());
            }
        };
        let serializer: Box<dyn CacheEntrySerializer> = match ext {
            "json" => Box::new(JsonCacheEntrySerializer),
            _ => {
                return Err(format!("Unsupported file extension: {ext}").into());
            }
        };
        Ok(serializer)
    }
}

impl Cache for FileCache {
    fn get_all_urls(&self) -> Vec<String> {
        self.contents.keys().cloned().collect()
    }
    fn get_all(&self, url: &str) -> Vec<&CacheEntry> {
        self.contents
            .get(url)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    fn insert(&mut self, url: String, text: String) {
        let entry = CacheEntry {
            created: Utc::now(),
            text,
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
        let removed = self
            .contents
            .remove(url)
            .unwrap_or_default();
        if removed.len() > 0 {
            log::info!(
                "Removed {} from cache which had {} entries",
                url,
                removed.len()
            );
            self.modified = true;
        }
        removed
    }
    fn save(&self) -> Result<()> {
        let path = &self.path.display();
        self.serializer
            .serialize(&self.contents)
            .and_then(|text| self.file_handler.save(&self.path, text))
            .inspect(|_| log::info!("Saved cache to {path}"))
            .inspect_err(|err| {
                log::error!("Error saving cache to {path}: {err}")
            })
    }
    fn size_bytes(&self) -> Result<u64> {
        if self.path.exists() {
            Ok(metadata(&self.path)?.size())
        } else {
            Ok(0)
        }
    }
    fn clear(&mut self) {
        self.contents.clear();
        self.modified = true;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::file_handler::MockFileHandler;

    use super::*;
    impl FileCache {
        fn for_test() -> Self {
            FileCache {
                contents:     HashMap::new(),
                file_handler: Box::new(MockFileHandler),
                max_entries:  None,
                modified:     false,
                path:         "/dev/null".into(),
                serializer:   Box::new(JsonCacheEntrySerializer),
            }
        }
    }

    #[test]
    fn test_insert_with_max_values() {
        let mut cache = FileCache::for_test();
        cache.max_entries = Some(2);
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
    }

    #[test]
    fn test_insert_with_no_max_values() {
        let mut cache = FileCache::for_test();
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
        let mut cache = FileCache::for_test();
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
        let mut cache = FileCache::for_test();
        cache.insert("url".into(), "text".into());
        cache.insert("url".into(), "text2".into());
        cache.insert("url2".into(), "text3".into());

        let removed = cache.remove("url");
        assert_eq!(removed.len(), 2);
        assert_eq!(cache.contents.len(), 1);
    }

    #[test]
    fn test_prune() {
        let mut cache = FileCache::for_test();
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
        let mut cache = FileCache::for_test();
        cache.contents = contents.clone();

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
        let mut cache = FileCache::for_test();
        assert_eq!(cache.modified, false);
        cache.insert("url".into(), "foo".into());
        assert_eq!(cache.modified, true);
    }
}
