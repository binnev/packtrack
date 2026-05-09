use crate::Result;
use crate::url_store::models::AnnotatedUrl;

pub trait UrlStore {
    /// Add an entry to the url store.
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()>;

    /// Remove any entries that match the given query. Return the entries that
    /// were removed.
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>>;

    /// Filter the contents of the url store by a query. If the query is none,
    /// return all the urls.
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl>;

    /// Save the URL store to preserve it between runs.
    /// `Result` so the implementation can do IO.
    fn save(&self) -> Result<()>;
}
