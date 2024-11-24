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
    fn new() -> Result<Self> {
        let cache_file = Self::get_file()?;
        let contents = utils::load_json(&cache_file)?;
        Ok(Self { contents })
    }
    fn get_file() -> Result<PathBuf> {
        Ok(get_cache_dir()?.join("packtrack-cache.json"))
    }
}
#[async_trait]
impl Cache for JsonCache {
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
        let cache_file = Self::get_file()?;
        utils::save_json(&cache_file, &self.contents)?;
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
mod test_cache {
    use super::*;

    pub struct TestCache {
        contents: HashMap<String, String>,
    }
    impl TestCache {
        fn new() -> Self {
            Self {
                contents: HashMap::new(),
            }
        }
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
