use crate::utils::UtcTime;
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
