use async_trait::async_trait;

use crate::{
    error::Result,
    tracker::{DhlTracker, GlsTracker, PostNLTracker},
};
use std::sync::Mutex;

use super::models::Package;

type Registry = Mutex<Vec<Box<dyn Fn() -> Box<dyn Tracker> + Send + Sync>>>;

// TODO: find a good mechanism for this
lazy_static::lazy_static! {
    static ref REGISTRY: Registry = Mutex::new(vec![
        Box::new(|| Box::new(PostNLTracker)),
        Box::new(|| Box::new(DhlTracker)),
        Box::new(|| Box::new(GlsTracker)),
    ]);
}

#[async_trait]
pub trait Tracker: Send + Sync {
    /// Lets caller code know whether the Tracker implementation is suitable for
    /// the given url, so that caller code can do dynamic dispatch.
    fn can_handle(&self, url: &str) -> bool;

    /// Track the package
    async fn track(&self, url: &str) -> Result<Package>;
}

/// Register the given Tracker implementation so that it can be selected
pub fn register(creator: Box<dyn Fn() -> Box<dyn Tracker> + Send + Sync>) {
    REGISTRY.lock().unwrap().push(creator);
}

/// Try to get a Tracker implementation for the given url.
pub fn get_handler(url: &str) -> Result<Box<dyn Tracker>> {
    for creator in REGISTRY
        .lock()
        .map_err(|err| format!("Error unlocking mutex: {err}"))?
        .iter()
    {
        let tracker = creator();
        if tracker.can_handle(url) {
            return Ok(tracker);
        }
    }

    Err(format!("Couldn't find a handler for {}", url).into())
}
