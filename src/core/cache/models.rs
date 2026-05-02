use std::time::Duration;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::tracker::TimeWindow;
use crate::utils::UtcTime;
use crate::{Result, utils};
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub text:    String,
    pub created: UtcTime,
}
impl CacheEntry {
    pub fn age(&self) -> TimeDelta {
        Utc::now() - self.created
    }
}
