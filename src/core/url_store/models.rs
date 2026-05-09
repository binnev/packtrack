use crate::utils::UtcTime;
use chrono::Utc;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;

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
