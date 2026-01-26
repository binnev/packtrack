use crate::Error;
use crate::Result;
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
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnnotatedUrl {
    pub url:         String,
    /// Sometimes URLs don't give you much context and it's easy to forget what
    /// the package is. Users can describe a URL here to remember that context.
    pub description: Option<String>,
    /// When the URL was added to the URL store.
    pub created:     Option<UtcTime>,
}
impl AnnotatedUrl {
    pub fn new(url: String, description: Option<String>) -> Self {
        Self {
            url,
            description,
            created: Some(Utc::now()),
        }
    }
}
impl Display for AnnotatedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)?;
        if let Some(d) = &self.description {
            write!(f, " ({d})")?;
        }
        Ok(())
    }
}

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URL store")]
    AlreadyInStore(String),

    #[display("'{_0}' was not found in the URL store")]
    NotFound(String),
}
impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Custom(e.to_string())
    }
}
