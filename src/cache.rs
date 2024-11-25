use std::{collections::HashMap, fs, path::PathBuf};

use crate::{settings, utils, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Cache {
    /// Get a cached response.text for the given URL
    /// `Option` because there may not be an entry for the URL.
    /// `Result` so the implementation can choose to do IO.
    async fn get(&self, url: &str) -> Result<Option<&str>>;

    /// Insert a cached response.text for the given URL.
    /// `mut` because the implementation must store its state in memory.
    /// `Result` so the implementation can choose to do IO.
    async fn insert(&mut self, url: String, text: String) -> Result<()>;

    /// Save the cache to preserve it between runs
    async fn save(&self) -> Result<()>;
}

#[derive(Default)]
pub struct JsonCache {
    // {URL: response.text}
    contents: HashMap<String, String>,
}
impl JsonCache {
    // RAII: load from file when instantiating
    pub fn new() -> Result<Self> {
        let cache_file = Self::get_file()?;
        let contents = utils::load_json(&cache_file)?;
        log::info!("Loaded JSON cache from {cache_file:?}");
        Ok(Self { contents })
    }
    fn get_file() -> Result<PathBuf> {
        Ok(get_cache_dir()?.join("packtrack-cache.json"))
    }
}
#[async_trait]
impl Cache for JsonCache {
    async fn get(&self, url: &str) -> Result<Option<&str>> {
        let entry = self
            .contents
            .get(url)
            .map(|s| s.as_str());

        match entry {
            Some(hit) => log::info!("Cache hit for {url}"),
            None => log::info!("Cache miss for {url}"),
        }
        Ok(entry)
    }
    async fn insert(&mut self, url: String, text: String) -> Result<()> {
        self.contents.insert(url.clone(), text);
        log::info!("Inserted new cache entry for {url}");
        Ok(())
    }
    // Save to file
    async fn save(&self) -> Result<()> {
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

/// In-memory cache implementation for testing
#[cfg(test)]
pub mod test_cache {
    use super::*;

    #[derive(Default)]
    pub struct TestCache {
        pub contents: HashMap<String, String>,
    }
    #[async_trait]
    impl Cache for TestCache {
        async fn get(&self, url: &str) -> Result<Option<&str>> {
            Ok(self
                .contents
                .get(url)
                .map(|s| s.as_str()))
        }
        async fn insert(&mut self, url: String, text: String) -> Result<()> {
            self.contents.insert(url, text);
            Ok(())
        }
        // Save to file
        async fn save(&self) -> Result<()> {
            Ok(()) // don't write to file in tests
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_cache::*;
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_sharing_cache() -> Result<()> {
        let mut cache = TestCache::default();
        let mutex = Mutex::new(cache);

        let ref1 = &mutex;
        let ref2 = &mutex;

        assert_eq!(ref1.lock().unwrap().contents, HashMap::new());

        ref1.lock()
            .unwrap()
            .insert("url".into(), "text".into())
            .await?;

        // the insert should succeed
        assert_eq!(
            ref1.lock().unwrap().contents,
            HashMap::from([("url".into(), "text".into())])
        );

        ref2.lock()
            .unwrap()
            .insert("url2".into(), "text2".into())
            .await?;

        assert_eq!(
            ref1.lock().unwrap().contents,
            HashMap::from([
                ("url".into(), "text".into()),
                ("url2".into(), "text2".into()),
            ])
        );

        Ok(())
    }
}
