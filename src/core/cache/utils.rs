use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::cache::models::CacheEntry;
use crate::tracker::TimeWindow;
use crate::utils::UtcTime;
use crate::{Result, utils};
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};

pub fn log_hit(url: &str, entry: &CacheEntry) {
    log::debug!(
        "Reusing {}s old cache entry for {url}",
        entry.age().num_seconds()
    )
}

/// Get the cache dir for the current OS
pub fn get_cache_dir() -> Result<PathBuf> {
    let dirs = utils::project_dirs()?;
    let cache_dir = dirs.cache_dir();
    Ok(cache_dir.to_owned())
}
