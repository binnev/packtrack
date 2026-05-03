use crate::Error;
use crate::Result;
use crate::url_store::models::AnnotatedUrl;
use crate::utils::UtcTime;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use std::{fs, path::PathBuf};

pub trait UrlStore {
    /// Add an entry to the url store. This should also persist the changes (to
    /// disk, or wherever the data is stored)
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()>;

    /// Remove any entries that match the given query. Return the entries that
    /// were removed. This should also persist the changes (to disk, or
    /// wherever the data is stored)
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>>;

    /// Filter the contents of the url store by a query. If the query is none,
    /// return all the urls.
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl>;

    /// Save the URL store to preserve it between runs.
    /// `Result` so the implementation can do IO.
    fn save(&self) -> Result<()>;
}
